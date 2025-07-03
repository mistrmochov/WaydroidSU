use colored::*;
use dbus::blocking::{Connection, Proxy};
use ini::configparser::ini::Ini;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

use crate::constants::WAYDROID_CONFIG;
use crate::{msg_err_str, msg_sub};

pub struct WaydroidContainer {
    conn: Connection,
    session: HashMap<String, String>,
}

impl WaydroidContainer {
    pub fn new() -> Result<Self, dbus::Error> {
        let conn = Connection::new_system()?;
        let proxy = conn.with_proxy(
            "id.waydro.Container",
            "/ContainerManager",
            Duration::from_millis(5000),
        );
        let (session,): (HashMap<String, String>,) =
            proxy.method_call("id.waydro.ContainerManager", "GetSession", ())?;
        Ok(Self { conn, session })
    }

    fn get_proxy(&self) -> Proxy<'_, &Connection> {
        let proxy = self.conn.with_proxy(
            "id.waydro.Container",
            "/ContainerManager",
            Duration::from_millis(5000),
        );
        proxy
    }

    pub fn get_session(&self) -> HashMap<String, String> {
        self.session.clone()
    }

    fn refresh_session(&mut self, store: bool) -> Result<(), dbus::Error> {
        let proxy = self.get_proxy();
        let (session,): (HashMap<String, String>,) =
            proxy.method_call("id.waydro.ContainerManager", "GetSession", ())?;
        if store {
            self.session = session;
        }
        Ok(())
    }

    pub fn is_session_running(&mut self, refresh: bool, store: bool) -> Result<bool, dbus::Error> {
        if refresh {
            self.refresh_session(store)?;
        }
        let mut running = false;
        let session = self.get_session();
        if !session.is_empty() {
            for (key, value) in session {
                if key == "state" && value == "FROZEN" {
                    self.unfreeze()?;
                }
            }
            running = true;
        }
        Ok(running)
    }

    pub fn is_container_running(&mut self) -> Result<bool, dbus::Error> {
        let mut running = false;
        if self.is_session_running(true, true)? {
            let session = self.get_session();
            for (key, value) in session {
                if key == "state" && value == "FROZEN" {
                    self.unfreeze()?;
                    running = true;
                } else if key == "state" && value == "RUNNING" {
                    running = true;
                }
            }
        }
        Ok(running)
    }

    pub fn stop(&mut self, session_stop: bool) -> Result<(), dbus::Error> {
        self.refresh_session(true)?;
        let proxy = self.get_proxy();
        let (): () = proxy.method_call("id.waydro.ContainerManager", "Stop", (session_stop,))?;
        Ok(())
    }

    pub fn unfreeze(&mut self) -> Result<(), dbus::Error> {
        let proxy = self.get_proxy();
        let (): () = proxy.method_call("id.waydro.ContainerManager", "Unfreeze", ())?;
        self.refresh_session(true)?;
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), dbus::Error> {
        if !self.is_session_running(false, false)? {
            return Err(dbus::Error::new_failed(
                "Can't start Waydroid container, because Waydroid session isn't running!",
            ));
        }
        let proxy = self.get_proxy();
        let session = self.get_session();
        let (): () = proxy.method_call("id.waydro.ContainerManager", "Start", (session,))?;
        self.refresh_session(true)?;

        Ok(())
    }

    pub fn restart_countdown(&mut self) -> Result<(), dbus::Error> {
        let secs = 5;
        for i in 0..=(secs - 1) {
            msg_sub(&format!(
                "Restarting Waydroid in {}",
                (secs - i).to_string().blue().bold()
            ));
            if i == (secs - 1) {
                self.stop(false)?;
                self.start()?;
            }
            sleep(Duration::from_secs(1));
        }
        Ok(())
    }
}

impl Clone for WaydroidContainer {
    fn clone(&self) -> Self {
        let conn = Connection::new_system().expect(&msg_err_str("Failed to get D-Bus connection"));
        Self {
            conn: conn,
            session: self.session.clone(),
        }
    }
}

pub fn has_overlay() -> anyhow::Result<bool, Box<dyn std::error::Error>> {
    let mut conf = Ini::new();
    conf.load(WAYDROID_CONFIG)?;
    Ok(conf
        .get("waydroid", "mount_overlays")
        .expect(&msg_err_str("Failed to read the config file."))
        == "True")
}
