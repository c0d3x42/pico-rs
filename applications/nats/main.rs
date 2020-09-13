pub fn start_nats() {
  let nc = nats::connect("localhost").unwrap();
  let sub = nc
    .subscribe("my.subject")
    .unwrap()
    .with_handler(move |msg| {
      println!("Received {}", &msg);
      Ok(())
    });
  nc.publish("my.subject", "Hello World!");
}
