#[test]
fn path_adding() {
    let current = std::env::current_dir().unwrap();
    let p1 = "./minecraft";

    let p2 = current.join(p1);
    dbg!(&p2);
    assert!(p2.exists());
}
