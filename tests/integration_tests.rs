#[cfg(test)]
#[path = "../src/main.rs"]
mod bloom;

mod integration {
    use dockertest::{Composition, DockerTest, Image};
    use std::sync::{Arc, Mutex};

    #[test]
    fn hello_world_test() {
        // Define our test instance
        let mut test = DockerTest::new();

        // Construct the Composition to be added to the test.
        // A Composition is an Image configured with environment, arguments, StartPolicy, etc.,
        // seen as an instance of the Image prior to constructing the Container.
        let hello = Composition::with_image(Image::with_repository("redis").tag("7"))
            .with_container_name("redis")
            .publish_all_ports()
            .to_owned();

        // Populate the test instance.
        // The order of compositions added reflect the execution order (depending on StartPolicy).
        test.add_composition(hello);
        let has_ran: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
        let has_redis_ran_test = has_ran.clone();
        test.run(|ops| async move {
            // A handle to operate on the Container.
            let _container = ops.handle("redis");

            // The container is in a running state at this point.
            // Depending on the Image, it may exit on its own (like this hello-world image)
            let mut ran = has_redis_ran_test.lock().unwrap();
            *ran = true;
        });

        let redis_ran = has_ran.lock().unwrap();

        assert!(*redis_ran);
    }
}
