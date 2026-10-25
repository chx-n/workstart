//! Read-only Windows APIs. These functions are intentionally not Tauri commands.
//! Invoke on a blocking worker; COM objects never escape their creating thread.
use super::*;
use std::{mem::size_of, path::Path};
use windows::{
    Win32::{
        Foundation::{CloseHandle, ERROR_NO_MORE_FILES, ERROR_NO_MORE_ITEMS, HANDLE},
        Graphics::Gdi::*,
        Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES,
        System::{
            Com::*, Diagnostics::ToolHelp::*, Environment::ExpandEnvironmentStringsW, Registry::*,
            Threading::*,
        },
        UI::{
            Shell::*,
            WindowsAndMessaging::{DI_NORMAL, DestroyIcon, DrawIconEx},
        },
    },
    core::{Interface, PCWSTR, PWSTR},
};

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
fn text(value: &[u16]) -> Option<String> {
    String::from_utf16(&value[..value.iter().position(|c| *c == 0).unwrap_or(value.len())]).ok()
}
struct Key(HKEY);
impl Drop for Key {
    fn drop(&mut self) {
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}
struct Handle(HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}
struct Apartment;
impl Apartment {
    fn new() -> Option<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok().ok()?;
        }
        Some(Self)
    }
}
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

fn open_key(parent: HKEY, path: &str, view: REG_SAM_FLAGS) -> Option<Key> {
    let mut key = HKEY::default();
    unsafe {
        RegOpenKeyExW(
            parent,
            PCWSTR(wide(path).as_ptr()),
            None,
            KEY_READ | view,
            &mut key,
        )
        .ok()
        .ok()?;
    }
    Some(Key(key))
}
fn registry_string(key: HKEY, name: &str) -> Option<String> {
    let name = wide(name);
    let mut kind = REG_VALUE_TYPE::default();
    let mut length = 0;
    unsafe {
        RegQueryValueExW(
            key,
            PCWSTR(name.as_ptr()),
            None,
            Some(&mut kind),
            None,
            Some(&mut length),
        )
        .ok()
        .ok()?;
        if ![REG_SZ, REG_EXPAND_SZ].contains(&kind) || length > 65536 || length % 2 != 0 {
            return None;
        }
        let mut buffer = vec![0u16; length as usize / 2 + 1];
        RegQueryValueExW(
            key,
            PCWSTR(name.as_ptr()),
            None,
            Some(&mut kind),
            Some(buffer.as_mut_ptr().cast()),
            Some(&mut length),
        )
        .ok()
        .ok()?;
        if ![REG_SZ, REG_EXPAND_SZ].contains(&kind) {
            return None;
        }
        let value = text(&buffer)?;
        if kind != REG_EXPAND_SZ {
            return Some(value);
        }
        let source = wide(&value);
        let mut expanded = vec![0u16; 32768];
        let count = ExpandEnvironmentStringsW(PCWSTR(source.as_ptr()), Some(&mut expanded));
        if count == 0 || count as usize > expanded.len() {
            return None;
        }
        text(&expanded)
    }
}
fn registrations(parent: HKEY, root: &str, view: REG_SAM_FLAGS, source: Source) -> Vec<Record> {
    let Some(key) = open_key(parent, root, view) else {
        return vec![];
    };
    let mut records = Vec::new();
    // Registry key names max out at 255 UTF-16 units. Cap enumeration defensively.
    for index in 0..16384 {
        let mut name = [0u16; 256];
        let mut length = name.len() as u32;
        let status = unsafe {
            RegEnumKeyExW(
                key.0,
                index,
                Some(PWSTR(name.as_mut_ptr())),
                &mut length,
                None,
                None,
                None,
                None,
            )
        };
        if status == ERROR_NO_MORE_ITEMS {
            break;
        }
        if status.is_err() {
            break;
        }
        let Some(name) = text(&name[..length as usize]) else {
            continue;
        };
        let Some(child) = open_key(key.0, &name, view) else {
            continue;
        };
        let (name, executable) = if matches!(source, Source::UserAppPaths | Source::MachineAppPaths)
        {
            (name, registry_string(child.0, ""))
        } else {
            let Some(name) = registry_string(child.0, "DisplayName") else {
                continue;
            };
            let executable = uninstall_executable(
                &name,
                registry_string(child.0, "DisplayIcon").as_deref(),
                registry_string(child.0, "InstallLocation").as_deref(),
            );
            (name, executable)
        };
        records.push(Record {
            source,
            name,
            executable,
            shortcut: None,
        });
    }
    records
}

fn known_folder(id: &windows::core::GUID) -> Option<String> {
    unsafe {
        let path = SHGetKnownFolderPath(id, KF_FLAG_DONT_VERIFY, None).ok()?;
        let result = path.to_string().ok();
        CoTaskMemFree(Some(path.0.cast()));
        result
    }
}
fn shortcuts(root: &Path, source: Source, depth: usize, records: &mut Vec<Record>) {
    if depth > 16 || records.len() > 16384 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        // Do not follow junctions, symbolic links, or other reparse points.
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            continue;
        }
        let path = entry.path();
        if metadata.is_dir() {
            shortcuts(&path, source, depth + 1, records);
        } else if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("lnk"))
        {
            if let (Some(name), Some(path)) =
                (path.file_stem().and_then(|s| s.to_str()), path.to_str())
            {
                records.push(Record {
                    source,
                    name: name.into(),
                    executable: None,
                    shortcut: Some(path.into()),
                });
            }
        }
        if records.len() > 16384 {
            break;
        }
    }
}

fn resolve_shortcut(path: &str) -> Option<String> {
    let _apartment = Apartment::new()?;
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
        let persist: IPersistFile = link.cast().ok()?;
        persist.Load(PCWSTR(wide(path).as_ptr()), STGM_READ).ok()?;
        let mut target = vec![0u16; 32768];
        // GetPath reads the file target. No Resolve call: no UI, tracking, search,
        // installer activation, or network resolution. UWP PIDLs yield no path.
        link.GetPath(&mut target, std::ptr::null_mut(), SLGP_RAWPATH.0 as u32)
            .ok()?;
        executable_path(&text(&target)?)
    }
}

fn canonical_file(path: &str) -> Option<String> {
    let path = executable_path(path)?;
    let path = std::fs::canonicalize(path).ok()?;
    if !path.is_file() {
        return None;
    }
    executable_path(path.to_str()?)
}

pub fn discover_installed() -> Vec<Application> {
    let mut records = Vec::new();
    for (root, user, machine) in [
        (
            r"Software\Microsoft\Windows\CurrentVersion\App Paths",
            Source::UserAppPaths,
            Source::MachineAppPaths,
        ),
        (
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall",
            Source::UserUninstall,
            Source::MachineUninstall,
        ),
    ] {
        for (hive, source) in [(HKEY_CURRENT_USER, user), (HKEY_LOCAL_MACHINE, machine)] {
            for view in [KEY_WOW64_64KEY, KEY_WOW64_32KEY] {
                records.extend(registrations(hive, root, view, source));
            }
        }
    }
    for (id, source) in [
        (FOLDERID_Programs, Source::UserShortcut),
        (FOLDERID_CommonPrograms, Source::CommonShortcut),
    ] {
        if let Some(path) = known_folder(&id) {
            shortcuts(Path::new(&path), source, 0, &mut records);
        }
    }
    let locations = known_locations(
        known_folder(&FOLDERID_LocalAppData).as_deref(),
        known_folder(&FOLDERID_ProgramFiles).as_deref(),
        known_folder(&FOLDERID_ProgramFilesX86).as_deref(),
    );
    discover(records, &locations, resolve_shortcut, canonical_file)
}

pub fn is_running(executable: &str) -> RunningState {
    let Some(executable) = canonical_file(executable) else {
        return RunningState::Unknown;
    };
    let mut processes = Vec::new();
    let mut complete = true;
    unsafe {
        let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).map(Handle) else {
            return RunningState::Unknown;
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut status = Process32FirstW(snapshot.0, &mut entry);
        while status.is_ok() {
            // PID 0/4 are Windows kernel processes, not candidate user apps.
            if entry.th32ProcessID > 4 {
                let path = OpenProcess(
                    PROCESS_QUERY_LIMITED_INFORMATION,
                    false,
                    entry.th32ProcessID,
                )
                .ok()
                .map(Handle)
                .and_then(|handle| {
                    let mut buffer = vec![0u16; 32768];
                    let mut size = buffer.len() as u32;
                    QueryFullProcessImageNameW(
                        handle.0,
                        PROCESS_NAME_WIN32,
                        PWSTR(buffer.as_mut_ptr()),
                        &mut size,
                    )
                    .ok()?;
                    canonical_file(&text(&buffer[..size as usize])?)
                });
                if path.is_none() {
                    complete = false;
                }
                processes.push(Process {
                    canonical_path: path,
                    name: text(&entry.szExeFile).unwrap_or_default(),
                });
            }
            status = Process32NextW(snapshot.0, &mut entry);
        }
        if status.is_err_and(|e| e.code() != ERROR_NO_MORE_FILES.to_hresult()) {
            complete = false;
        }
    }
    running_state(&executable, &processes, complete)
}

pub fn launch(plan: &DirectLaunch) -> Result<u32, ErrorCategory> {
    let executable = canonical_file(plan.executable()).ok_or(ErrorCategory::NotFound)?;
    let child = std::process::Command::new(executable)
        .args(plan.arguments())
        .spawn()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::PermissionDenied => ErrorCategory::PermissionDenied,
            std::io::ErrorKind::NotFound => ErrorCategory::NotFound,
            _ => ErrorCategory::LaunchFailed,
        })?;
    Ok(child.id())
}

pub fn application_icon(path: &str) -> Icon {
    lookup_icon(path, extract_icon)
}

fn extract_icon(path: &str) -> Option<RgbaIcon> {
    let path = canonical_file(path)?;
    let _apartment = Apartment::new()?;
    unsafe {
        let mut info = SHFILEINFOW::default();
        if SHGetFileInfoW(
            PCWSTR(wide(&path).as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut info),
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        ) == 0
        {
            return None;
        }
        let result = (|| {
            let dc = CreateCompatibleDC(None);
            if dc.is_invalid() {
                return None;
            }
            let mut bits = std::ptr::null_mut();
            let bitmap_info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: 32,
                    biHeight: -32,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let bitmap = match CreateDIBSection(
                Some(dc),
                &bitmap_info,
                DIB_RGB_COLORS,
                &mut bits,
                None,
                0,
            ) {
                Ok(bitmap) => bitmap,
                Err(_) => {
                    let _ = DeleteDC(dc);
                    return None;
                }
            };
            let old = SelectObject(dc, HGDIOBJ(bitmap.0));
            std::ptr::write_bytes(bits, 0, 32 * 32 * 4);
            let drawn = DrawIconEx(dc, 0, 0, info.hIcon, 32, 32, 0, None, DI_NORMAL).is_ok();
            let mut pixels = std::slice::from_raw_parts(bits.cast::<u8>(), 32 * 32 * 4).to_vec();
            SelectObject(dc, old);
            let _ = DeleteObject(HGDIOBJ(bitmap.0));
            let _ = DeleteDC(dc);
            if !drawn {
                return None;
            }
            for pixel in pixels.chunks_exact_mut(4) {
                pixel.swap(0, 2);
            }
            Some(RgbaIcon {
                width: 32,
                height: 32,
                pixels,
            })
        })();
        let _ = DestroyIcon(info.hIcon);
        result
    }
}
