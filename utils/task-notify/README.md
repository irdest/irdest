# task-notify

A wrapper which wakes tasks on mutable accesses to the wrapped value.

This can be used to transparently notify an asyncronous task that it
should, for example, check for more work in a queue or try again to
acquire a lock.

```rust
let c = Arc::new(Mutex::new(Notify::new(VecDeque::new())));
```

```
// task 1
async fn task_1() {
  let mut mg = c.lock().await;
  mg.push_back(42); // access via DerefMut
}
```

```
// task 2
impl Future for Task2 {
    type Output = u64;
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut lock = Box::pin(t.lock());
        match Pin::new(&mut lock).poll(cx) {
            Poll::Ready(ref mut mg) => match mg.pop_front() {
                Some(v) => Poll::Ready(v),
                None => {
                    // Insert a Waker when we can't make progress for the first time!
                    Notify::add_waker(mg, cx.waker().clone());
                    Poll::Pending
                }
            },
            p => p, // The Mutex will wake us if we can make progress
        }
    }
}
```
