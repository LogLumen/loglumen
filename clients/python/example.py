import math
import random
import time

import loglumen as ll

ll.init()

initial_loss = random.uniform(2.0, 3.0)
decay = random.uniform(1.5, 2.5)
peak_lr = random.uniform(1e-4, 5e-4)

for step in range(10):
    progress = step / 10
    loss = initial_loss * math.exp(-decay * progress)
    ll.log(
        "train",
        step=step,
        loss=loss * random.uniform(0.95, 1.05),
        learning_rate=peak_lr * (1 - progress),
        grad_norm=random.uniform(0.8, 1.2) / math.sqrt(1 + step),
    )
    if step % 5 == 4:
        ll.log(
            "eval",
            val_loss=loss * random.uniform(1.05, 1.2),
            accuracy=50 + 40 * progress + random.uniform(-3.0, 3.0),
        )
ll.log("eval", accuracy=90)

# Give the writer a moment to drain the queue before exiting.
time.sleep(0.5)
