use loglumen as ll;
use rand::random_range;
use std::time::Duration;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    ll::Config::default().init().await;

    let initial_loss = random_range(2.0..3.0);
    let decay = random_range(1.5..2.5);
    let peak_lr = random_range(1e-4..5e-4);

    for step in 0..10 {
        let progress = step as f64 / 10.0;
        let loss = initial_loss * (-decay * progress).exp();
        ll::log!(
            "train",
            step = step,
            loss = loss * random_range(0.95..1.05),
            learning_rate = peak_lr * (1.0 - progress),
            grad_norm = random_range(0.8..1.2) / (1.0 + step as f64).sqrt(),
        );
        if step % 5 == 4 {
            ll::log!(
                "eval",
                val_loss = loss * random_range(1.05..1.2),
                accuracy = 50.0 + 40.0 * progress + random_range(-3.0..3.0),
            );
        }
    }
    ll::log("eval", ll::Payload(vec![("accuracy", 90.into())]));

    // Give the writer a moment to drain the queue before exiting.
    tokio::time::sleep(Duration::from_millis(500)).await;
}
