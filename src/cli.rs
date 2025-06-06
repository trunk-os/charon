use crate::{CompiledPackage, CompiledSource};
use anyhow::{anyhow, Result};
use std::path::PathBuf;

const PODMAN_COMMAND: &str = "podman";
const QEMU_COMMAND: &str = "qemu-system-x86_64";

pub fn generate_command(package: CompiledPackage, volume_root: PathBuf) -> Result<Vec<String>> {
    match package.source {
        CompiledSource::HTTP(_) => generate_vm_command(&package, &volume_root),
        CompiledSource::Container(_) => generate_container_command(&package, &volume_root),
    }
}

pub fn generate_vm_command(
    _package: &CompiledPackage,
    _volume_root: &PathBuf,
) -> Result<Vec<String>> {
    Ok(vec![QEMU_COMMAND.to_string()])
}

pub fn generate_container_command(
    package: &CompiledPackage,
    volume_root: &PathBuf,
) -> Result<Vec<String>> {
    let mut cmd = vec![PODMAN_COMMAND.into()];
    let name = format!("{}-{}", package.title.name, package.title.version);
    cmd.append(&mut vec!["--name".into(), name]);

    if let Some(hostname) = &package.networking.hostname {
        cmd.append(&mut vec!["--hostname".into(), hostname.clone()]);
    }

    // FIXME: solve creating this network in advance
    if let Some(internal_network) = &package.networking.internal_network {
        cmd.append(&mut vec!["--network".into(), internal_network.clone()]);
    }

    for (hostport, localport) in &package.networking.forward_ports {
        let portmap = format!("{}:{}", hostport, localport);
        cmd.append(&mut vec!["-p".into(), portmap]);
    }

    // FIXME: uPnP
    for (hostport, localport) in &package.networking.expose_ports {
        let portmap = format!("{}:{}", hostport, localport);
        cmd.append(&mut vec!["-p".into(), portmap]);
    }

    for volume in &package.storage.volumes {
        // FIXME: create filesystems on block devices.
        //        ignoring them for now
        if let Some(mountpoint) = &volume.mountpoint {
            let volmap = if !volume.private {
                format!(
                    "{}/{}:{}:rshared",
                    volume_root.display(),
                    volume.name,
                    mountpoint
                )
            } else {
                format!(
                    "{}/{}:{}:rprivate",
                    volume_root.display(),
                    volume.name,
                    mountpoint
                )
            };
            cmd.append(&mut vec!["-v".into(), volmap]);
        }
    }

    let name = if let CompiledSource::Container(name) = &package.source {
        name
    } else {
        return Err(anyhow!("Genuinely curious how you got here, not gonna lie"));
    };

    if package.system.host_pid {
        cmd.append(&mut vec!["--pid".into(), "host".into()]);
    }

    // FIXME: check for this conflict in validate
    if package.system.host_net && package.networking.internal_network.is_none() {
        cmd.append(&mut vec!["--network".into(), "host".into()]);
    }

    if package.system.privileged {
        cmd.append(&mut vec!["--privileged".into()]);
    }

    for cap in &package.system.capabilities {
        cmd.append(&mut vec!["--cap-add".into(), cap.into()]);
    }

    cmd.append(&mut vec!["-d".into(), name.into()]);

    Ok(cmd)
}

#[cfg(test)]
mod tests {
    #[allow(unused)]
    use crate::*;
    use anyhow::Result;

    fn string_vec(v: Vec<&str>) -> Vec<String> {
        v.iter().map(ToString::to_string).collect::<Vec<String>>()
    }

    fn load(registry: &Registry, name: &str, version: &str) -> Result<CompiledPackage> {
        registry.load(name, version)?.compile(&[])
    }

    #[test]
    fn podman_cli() {
        let registry = Registry::new("testdata/registry".into());
        assert_eq!(
            generate_command(
                load(&registry, "plex", "0.0.2").unwrap(),
                "/volume-root".into()
            )
            .unwrap(),
            string_vec(vec!["podman", "--name", "plex-0.0.2", "-d", "scratch"])
        );
        assert_eq!(
            generate_command(
                load(&registry, "plex", "0.0.1").unwrap(),
                "/volume-root".into()
            )
            .unwrap(),
            string_vec(vec!["podman", "--name", "plex-0.0.1", "-d", "scratch"])
        );
        assert_eq!(
            generate_command(
                load(&registry, "podman-test", "0.0.1").unwrap(),
                "/volume-root".into()
            )
            .unwrap(),
            string_vec(vec![
                "podman",
                "--name",
                "podman-test-0.0.1",
                "-v",
                "/volume-root/private:/private-test:rprivate",
                "-v",
                "/volume-root/shared:/shared-test:rshared",
                "--pid",
                "host",
                "--network",
                "host",
                "--privileged",
                "--cap-add",
                "SYS_ADMIN",
                "-d",
                "debian"
            ])
        );
    }
}
