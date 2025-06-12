use crate::{CompiledPackage, CompiledSource};
use anyhow::{anyhow, Result};
use std::path::PathBuf;

const PODMAN_COMMAND: &str = "podman";
const QEMU_COMMAND: &str = "qemu-system-x86_64";
const QEMU_IMAGE_FILENAME: &str = "image";
const QEMU_MONITOR_FILENAME: &str = "qemu-monitor";

pub fn generate_command(package: CompiledPackage, volume_root: PathBuf) -> Result<Vec<String>> {
    match package.source {
        CompiledSource::HTTP(_) => generate_vm_command(&package, &volume_root),
        CompiledSource::Container(_) => generate_container_command(&package, &volume_root),
    }
}

pub fn generate_vm_command(
    package: &CompiledPackage,
    volume_root: &PathBuf,
) -> Result<Vec<String>> {
    let mut cmd = vec![QEMU_COMMAND.to_string()];

    let mut fwdrules = String::new();
    for (host, guest) in &package.networking.forward_ports {
        fwdrules.push_str(&format!(",hostfwd=tcp:0.0.0.0:{}-:{}", host, guest));
    }

    for (host, guest) in &package.networking.expose_ports {
        fwdrules.push_str(&format!(",hostfwd=tcp:0.0.0.0:{}-:{}", host, guest));
    }

    cmd.append(&mut vec![
        "-nodefaults".into(),
        "-chardev".into(),
        format!(
            "socket,server=on,wait=off,id=char0,path={}",
            volume_root.join(QEMU_MONITOR_FILENAME).display(),
        ),
        "-mon".into(),
        "chardev=char0,mode=control,pretty=on".into(),
        "-machine".into(),
        "accel=kvm".into(),
        "-vga".into(),
        "none".into(), // FIXME: move to VNC
        "-m".into(),
        format!("{}M", package.resources.memory),
        "-cpu".into(),
        "max".into(),
        "-smp".into(),
        format!(
            "cpus={},cores={},maxcpus={}",
            package.resources.cpus, package.resources.cpus, package.resources.cpus
        ),
        "-nic".into(),
        format!("user{}", fwdrules),
    ]);

    cmd.push("-drive".into());
    cmd.push(format!(
        "driver=raw,if=virtio,file={},cache=none,media=disk,index={}",
        volume_root.join(QEMU_IMAGE_FILENAME).display(),
        // NOTE: this offsets the counter below for volumes
        0,
    ));

    let excluded_names = vec![QEMU_IMAGE_FILENAME, QEMU_MONITOR_FILENAME];

    for (x, volume) in package.storage.volumes.iter().enumerate() {
        if excluded_names.contains(&volume.name.as_str()) {
            return Err(anyhow!(
                "VM volumes cannot be named '{}'",
                // this outputs "'foo', or 'bar', or 'baz'"
                excluded_names.join("', or '")
            ));
        }

        cmd.push("-drive".to_string());
        cmd.push(format!(
            "driver=raw,if=virtio,file={},cache=none,media=disk,index={}",
            // FIXME formalize making these into files; this doesn't work right yet
            volume_root.join(&volume.name).display(),
            // NOTE: the first drive is above, which is the VM image, which is why this is offset.
            x + 1,
        ));
    }

    Ok(cmd)
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
                    "{}:{}:rshared",
                    volume_root.join(&volume.name).display(),
                    mountpoint
                )
            } else {
                format!(
                    "{}:{}:rprivate",
                    volume_root.join(&volume.name).display(),
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

    // TODO: cgroups

    cmd.append(&mut vec!["-d".into(), name.into()]);

    Ok(cmd)
}

#[cfg(test)]
mod tests {
    use crate::{
        cli::{PODMAN_COMMAND, QEMU_COMMAND},
        *,
    };
    use anyhow::Result;

    fn string_vec(v: Vec<&str>) -> Vec<String> {
        v.iter().map(ToString::to_string).collect::<Vec<String>>()
    }

    fn load(registry: &Registry, name: &str, version: &str) -> Result<CompiledPackage> {
        registry.load(name, version)?.compile()
    }

    #[test]
    fn qemu_cli() {
        let registry = Registry::new("testdata/registry".into());
        assert_eq!(
            generate_command(
                load(&registry, "plex-qemu", "0.0.2").unwrap(),
                "/volume-root".into()
            )
            .unwrap(),
            string_vec(vec![
                QEMU_COMMAND,
                "-nodefaults",
                "-chardev",
                "socket,server=on,wait=off,id=char0,path=/volume-root/qemu-monitor",
                "-mon",
                "chardev=char0,mode=control,pretty=on",
                "-machine",
                "accel=kvm",
                "-vga",
                "none",
                "-m",
                "8192M",
                "-cpu",
                "max",
                "-smp",
                "cpus=4,cores=4,maxcpus=4",
                "-nic",
                "user",
                "-drive",
                "driver=raw,if=virtio,file=/volume-root/image,cache=none,media=disk,index=0",
                "-drive",
                "driver=raw,if=virtio,file=/volume-root/test,cache=none,media=disk,index=1"
            ]),
        );

        assert_eq!(
            generate_command(
                load(&registry, "plex-qemu", "0.0.1").unwrap(),
                "/volume-root".into()
            )
            .unwrap(),
            string_vec(vec![
                QEMU_COMMAND,
                "-nodefaults",
                "-chardev",
                "socket,server=on,wait=off,id=char0,path=/volume-root/qemu-monitor",
                "-mon",
                "chardev=char0,mode=control,pretty=on",
                "-machine",
                "accel=kvm",
                "-vga",
                "none",
                "-m",
                "4096M",
                "-cpu",
                "max",
                "-smp",
                "cpus=8,cores=8,maxcpus=8",
                "-nic",
                "user,hostfwd=tcp:0.0.0.0:1234-:5678,hostfwd=tcp:0.0.0.0:2345-:6789",
                "-drive",
                "driver=raw,if=virtio,file=/volume-root/image,cache=none,media=disk,index=0"
            ]),
        );
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
            string_vec(vec![
                PODMAN_COMMAND,
                "--name",
                "plex-0.0.2",
                "-d",
                "scratch"
            ])
        );
        assert_eq!(
            generate_command(
                load(&registry, "plex", "0.0.1").unwrap(),
                "/volume-root".into()
            )
            .unwrap(),
            string_vec(vec![
                PODMAN_COMMAND,
                "--name",
                "plex-0.0.1",
                "-d",
                "scratch"
            ])
        );
        assert_eq!(
            generate_command(
                load(&registry, "podman-test", "0.0.1").unwrap(),
                "/volume-root".into()
            )
            .unwrap(),
            string_vec(vec![
                PODMAN_COMMAND,
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
