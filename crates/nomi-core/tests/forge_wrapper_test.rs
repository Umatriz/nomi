use tokio::process::Command;

#[tokio::test]
async fn forge_wrapper_test() {
    let mut cmd = Command::new("java")
        .arg("-jar")
        .arg("-Dforgewrapper.librariesDir=./minecraft/libraries")
        .arg("-Dforgewrapper.installer=./forge-1.20-46.0.14-installer.jar")
        .arg("-Dforgewrapper.minecraft=./minecraft/instances/1.20/1.20.jar")
        .arg("ForgeWrapperConverter-1.5.6-LOCAL.jar")
        .arg("--installer=./forge-1.20-46.0.14-installer.jar")
        .spawn()
        .unwrap();

    let status = cmd.wait().await.unwrap().code();
    dbg!(status);
}
