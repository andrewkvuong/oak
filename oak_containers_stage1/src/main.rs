//
// Copyright 2023 The Project Oak Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

#![feature(never_type)]

mod client;
mod image;

use anyhow::Context;
use clap::Parser;
use client::LauncherClient;
use nix::{
    mount::{mount, umount2, MntFlags, MsFlags},
    unistd::chroot,
};
use std::{
    error::Error,
    fs::{self, create_dir},
    path::Path,
};
use tokio::process::Command;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t = 2)]
    launcher_vsock_cid: u32,

    #[arg(long, default_value_t = 8080)]
    launcher_vsock_port: u32,

    #[arg(default_value = "/sbin/init")]
    init: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    if !Path::new("/dev").try_exists()? {
        create_dir("/dev").context("error creating /dev")?;
    }
    mount(
        None::<&str>,
        "/dev",
        Some("devtmpfs"),
        MsFlags::empty(),
        None::<&str>,
    )
    .context("error mounting /dev")?;

    Command::new("/mke2fs")
        .args(["/dev/ram0"])
        .spawn()?
        .wait()
        .await?;
    if !Path::new("/rootfs").try_exists()? {
        create_dir("/rootfs").context("error creating /rootfs")?;
    }
    mount(
        Some("/dev/ram0"),
        "/rootfs",
        Some("ext4"),
        MsFlags::empty(),
        None::<&str>,
    )
    .context("error mounting ramdrive to /rootfs")?;
    umount2("/dev", MntFlags::empty()).context("error unmounting /dev")?;

    chroot("/rootfs").context("error chrooting to /rootfs")?;
    // musl expects /proc to be mounted, otherwise libc calls such as realpath(2) will fail.
    create_dir("/proc").context("error creating /proc")?;
    mount(
        None::<&str>,
        "/proc",
        Some("proc"),
        MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID | MsFlags::MS_NODEV,
        None::<&str>,
    )
    .context("error mounting /proc")?;

    let mut client = LauncherClient::new(args.launcher_vsock_cid, args.launcher_vsock_port)
        .await
        .context("error creating the launcher client")?;

    image::load(&mut client, Path::new("/"))
        .await
        .context("error loading the system image")?;

    // If the image didn't contain a `/etc/machine-id` file, create a placeholder one that systemd
    // will replace during startup. If you don't have that file at all, `systemd-machine-id-setup`
    // unit will fail.
    if !Path::new("/etc/machine-id").exists() {
        fs::write("/etc/machine-id", []).context("error writing placeholder /etc/machine-id")?;
    }
    image::switch(&args.init).context("error switching to the system image")?
}
