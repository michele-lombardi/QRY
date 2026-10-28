//! Allowlisted external navigation, including cold starts and permission gating.

use std::sync::Mutex;
use tauri::{AppHandle, Manager};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Destination {
    Today,
    Statistics,
    Settings,
}

#[derive(Default)]
pub(crate) struct NavigationRuntime(Mutex<Option<Destination>>);

fn destination(url: &tauri::Url) -> Option<Destination> {
    if url.scheme() != "qry"
        || url.host_str() != Some("open")
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return None;
    }
    match url.path() {
        "/today" => Some(Destination::Today),
        "/statistics" => Some(Destination::Statistics),
        "/settings" => Some(Destination::Settings),
        _ => None,
    }
}

pub(crate) fn open_url(app: &AppHandle, url: &tauri::Url) {
    let Some(target) = destination(url) else {
        return;
    };
    *app.state::<NavigationRuntime>()
        .0
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = Some(target);
    deliver(app);
}

pub(crate) fn deliver(app: &AppHandle) {
    if !app
        .state::<crate::permission_flow::PermissionFlowRuntime>()
        .permits_normal_start()
    {
        crate::permission_flow::focus_primary_surface(app);
        return;
    }
    let target = app
        .state::<NavigationRuntime>()
        .0
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take();
    match target {
        Some(Destination::Today) => crate::shell::open_today_window(app.clone()),
        Some(Destination::Statistics) => crate::shell::open_statistics_window(app.clone()),
        Some(Destination::Settings) => crate::shell::open_settings_window(app.clone()),
        None => (),
    }
}

pub(crate) fn second_instance(app: &AppHandle, args: Vec<String>) {
    if let Some(url) = args
        .iter()
        .filter_map(|value| tauri::Url::parse(value).ok())
        .find(|url| destination(url).is_some())
    {
        open_url(app, &url);
    } else {
        crate::permission_flow::focus_primary_surface(app);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_routes_only_navigate_to_known_surfaces() {
        for (url, target) in [
            ("qry://open/today", Destination::Today),
            ("qry://open/statistics", Destination::Statistics),
            ("qry://open/settings", Destination::Settings),
        ] {
            assert_eq!(destination(&tauri::Url::parse(url).unwrap()), Some(target));
        }
        for url in [
            "https://open/today",
            "qry://open/quit",
            "qry://open/today?command=reset",
            "qry://user@open/today",
            "qry://open/today#x",
            "qry://other/today",
        ] {
            assert_eq!(destination(&tauri::Url::parse(url).unwrap()), None);
        }
    }
}
