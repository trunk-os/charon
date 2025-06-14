use super::*;
use crate::*;
use anyhow::Result;

fn string_vec(v: Vec<&str>) -> Vec<String> {
    v.iter().map(ToString::to_string).collect::<Vec<String>>()
}

fn load(registry: &Registry, name: &str, version: &str) -> Result<CompiledPackage> {
    registry.load(name, version)?.compile()
}

#[cfg(feature = "livetests")]
mod livetests {
    use tempfile::TempDir;

    use super::*;

    //#[test]
    // FIXME: this test cannot be run until I can figure out how file:// urls work in curl.
    // see: https://github.com/alexcrichton/curl-rust/issues/611
    #[allow(dead_code)]
    fn test_downloader() {
        let registry = Registry::new("testdata/registry".into());
        let pkg = load(&registry, "plex-qemu", "0.0.2").unwrap();
        let td = TempDir::new().unwrap();
        let path = td.path();
        download_vm_image(&pkg, path).unwrap();
        assert!(std::fs::exists(path.join(QEMU_IMAGE_FILENAME)).unwrap());
    }

    //
    // #[test]
    // fn launch_qemu() {
    //     let registry = Registry::new("testdata/registry".into());
    //     let args = generate_command(
    //         load(&registry, "plex-qemu", "0.0.2").unwrap(),
    //         "testdata/volume-root".into(),
    //     )
    //     .unwrap();
    //     let child = std::process::Command::new(&args[0])
    //         .args(args.iter().skip(1))
    //         .spawn();
    // }
}

mod cli_generation {
    use super::*;

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
