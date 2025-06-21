use crate::CompiledPackage;
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use tracing::info;
use zbus_systemd::{systemd1::ManagerProxy, zbus::Connection};

const SYSTEMD_SERVICE_ROOT: &str = "/etc/systemd/system";

const UNIT_TEMPLATE: &str = r#"
[Unit]
Description=Charon launcher for @PACKAGE_NAME@, version @PACKAGE_VERSION@
After= # FIXME: this needs to follow the trunk microservices boot

[Service]
ExecStart=/usr/bin/charon -r @REGISTRY_PATH@ launch @PACKAGE_NAME@ @PACKAGE_VERSION@ @VOLUME_ROOT@
ExecStop=/usr/bin/charon -r @REGISTRY_PATH@ stop @PACKAGE_NAME@ @PACKAGE_VERSION@ @VOLUME_ROOT@
Restart=always

[Install]
Alias=@PACKAGE_FILENAME@.service
"#;

pub async fn reload_systemd() -> Result<()> {
    let mgr = ManagerProxy::new(&Connection::system().await?).await?;
    mgr.reload().await?;

    info!("Reloaded systemd");
    Ok(())
}

#[derive(Debug, Clone)]
pub struct SystemdUnit {
    package: CompiledPackage,
}

impl SystemdUnit {
    pub fn new(package: CompiledPackage) -> Self {
        Self { package }
    }

    pub fn filename(&self) -> PathBuf {
        format!("{}/{}.service", SYSTEMD_SERVICE_ROOT, self.package.title).into()
    }

    pub fn unit(&self, registry_path: PathBuf, volume_root: PathBuf) -> Result<String> {
        let mut out = String::new();
        let mut variable = String::new();
        let mut in_variable = false;

        for ch in UNIT_TEMPLATE.chars() {
            if ch == '@' {
                in_variable = if in_variable {
                    match variable.as_str() {
                        "PACKAGE_NAME" => out.push_str(&self.package.title.name),
                        "PACKAGE_VERSION" => out.push_str(&self.package.title.version),
                        "PACKAGE_FILENAME" => out.push_str(&self.package.title.to_string()),
                        "VOLUME_ROOT" => out.push_str(volume_root.to_str().unwrap_or_default()),
                        "REGISTRY_PATH" => out.push_str(registry_path.to_str().unwrap_or_default()),
                        _ => return Err(anyhow!("invalid template variable '{}'", variable)),
                    };
                    variable = String::new();

                    false
                } else {
                    true
                }
            } else if in_variable {
                variable.push(ch)
            } else {
                out.push(ch)
            }
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use crate::{CompiledPackage, Registry};
    use anyhow::Result;
    use tempfile::TempDir;

    use super::SystemdUnit;

    fn load(registry: &Registry, name: &str, version: &str) -> Result<CompiledPackage> {
        registry.load(name, version)?.compile()
    }

    #[test]
    fn unit_filename() {
        let registry = Registry::new("testdata/registry".into());
        let unit = SystemdUnit::new(load(&registry, "podman-test", "0.0.2").unwrap());
        assert_eq!(
            unit.filename().as_os_str(),
            "/etc/systemd/system/podman-test-0.0.2.service"
        );
    }

    #[test]
    fn unit_contents() {
        let registry = Registry::new("testdata/registry".into());
        let td = TempDir::new().unwrap();
        let path = td.path();
        let pkg = load(&registry, "podman-test", "0.0.2").unwrap();
        let unit = SystemdUnit::new(pkg);
        let text = unit
            .unit("testdata/registry".into(), path.to_path_buf())
            .unwrap();
        assert_eq!(
            text,
            format!(
                r#"
[Unit]
Description=Charon launcher for podman-test, version 0.0.2
After= # FIXME: this needs to follow the trunk microservices boot

[Service]
ExecStart=/usr/bin/charon -r testdata/registry launch podman-test 0.0.2 {}
ExecStop=/usr/bin/charon -r testdata/registry stop podman-test 0.0.2 {}
Restart=always

[Install]
Alias=podman-test-0.0.2.service
"#,
                path.display(),
                path.display()
            )
        );
    }
}
