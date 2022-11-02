use photonio::task;

#[photonio::test]
async fn yield_now() {
    task::yield_now().await;
}
