use std::pin::Pin;
use std::time::Duration;

use async_io::Timer;
use async_stream::stream;
use futures::stream::{self, Stream, StreamExt};

// // Define custom error enums
// #[derive(Debug, thiserror::Error)]
// pub enum StreamErrorInt {
//     #[error("An integer error occurred: {0}")]
//     IntegerError(String),
// }

// #[derive(Debug, thiserror::Error)]
// pub enum StreamErrorString {
//     #[error("A string error occurred: {0}")]
//     StringError(String),
// }

#[uniffi_dart::export_stream(i32)]
pub fn simple_stream() -> impl Stream<Item = i32> {
    stream::iter(0..5)
}

#[uniffi_dart::export_stream(i32)]
pub fn count_stream() -> Pin<Box<dyn Stream<Item = i32> + Send>> {
    Box::pin(stream::iter(0..5))
}

#[uniffi_dart::export_stream(u64)]
pub fn fibonacci_stream() -> Pin<Box<dyn Stream<Item = u64> + Send>> {
    Box::pin(stream! {
        let mut a = 0;
        let mut b = 1;
        loop {
            yield a;
            let next = a + b;
            a = b;
            b = next;
        }
    })
}

#[uniffi_dart::export_stream(u64)]
pub fn async_timer_stream() -> Pin<Box<dyn Stream<Item = u64> + Send>> {
    Box::pin(stream! {
        let mut count = 0;
        loop {
            Timer::after(Duration::from_secs(1)).await;
            count += 1;
            yield count;
        }
    })
}

#[uniffi_dart::export_stream(String)]
pub fn combined_streams() -> impl Stream<Item = String> + Send {
    let stream1 = count_stream().take(5).map(|n| format!("Count: {}", n));
    let stream3 = fibonacci_stream().take(5).map(|n| format!("Fibonacci: {}", n));

    stream::select(stream1, stream3)
}

// pub fn error_stream() -> impl Stream<Item = Result<i32,StreamErrorInt> > +Send {
//     {
//         let(mut __yield_tx,__yield_rx) = unsafe {
//             async_stream::__private::yielder::pair()
//         };
//         async_stream::__private::AsyncStream::new(__yield_rx,async move {
//             __yield_tx.send(Ok(1)).await;
//             __yield_tx.send(Ok(2)).await;
//             __yield_tx.send(Err(StreamErrorInt::IntegerError("An error occurred".to_string()))).await;
//             __yield_tx.send(Ok(4)).await;
//         })
//     }
// }
// #[derive(uniffi::Object)]
// pub struct ErrorStreamStreamExt {
//     stream:tokio::sync::Mutex<std::pin::Pin<Box<dyn futures::Stream<Item = Result<i32,StreamErrorInt> > +Send>> > ,
// }
// impl ErrorStreamStreamExt {
//     pub fn new() -> std::sync::Arc<Self>{
//         std::sync::Arc::new(Self {
//             stream:tokio::sync::Mutex::new(Box::pin(error_stream())),
//         })
//     }
//     pub async fn next(&self) -> Option<Result<i32,StreamErrorInt> >{
//         let mut stream = self.stream.lock().await;
//         stream.as_mut().next().await
//     }

// }

// #[uniffi_dart::export_stream(Result<i32, StreamErrorInt>)]
// pub fn error_stream() -> impl Stream<Item = Result<i32, StreamErrorInt>> + Send {
//     stream! {
//         yield Ok(1);
//         yield Ok(2);
//         yield Err("An error occurred".to_string());
//         yield Ok(4);
//     }
// }

// #[uniffi_dart::export_stream(Result<i32, StreamErrorString>)]
// pub fn combined_error_streams() -> impl Stream<Item = Result<i32, StreamErrorString>> + Send {
//     let stream1 = count_stream()
//         .take(3)
//         .map(|n| Ok(format!("Count: {}", n)));
//     let stream3 = fibonacci_stream()
//         .take(3)
//         .map(|n| {
//             if n == 2 {
//                 Err("Fibonacci error".to_string())
//             } else {
//                 Ok(format!("Fibonacci: {}", n))
//             }
//         });

//     stream::select(stream1, stream3)
// }

#[cfg(test)]
mod tests {
    use async_io::block_on;
    use futures::stream::StreamExt;

    use super::*;

    #[test]
    fn test_simple_stream() {
        block_on(async {
            let result: Vec<i32> = simple_stream().collect().await;
            assert_eq!(result, vec![0, 1, 2, 3, 4]);
        });
    }

    #[test]
    fn test_count_stream() {
        block_on(async {
            let result: Vec<i32> = count_stream().collect().await;
            assert_eq!(result, vec![0, 1, 2, 3, 4]);
        });
    }

    // #[tokio::test]
    // async fn test_alphabet_stream() {
    //     let result: String = alphabet_stream().collect().await;
    //     assert_eq!(result, "ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    // }

    #[test]
    fn test_fibonacci_stream() {
        block_on(async {
            let result: Vec<u64> = fibonacci_stream().take(10).collect().await;
            assert_eq!(result, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
        });
    }

    #[test]
    fn test_async_timer_stream() {
        block_on(async {
            let mut stream = async_timer_stream();
            let mut values = Vec::new();
            for _ in 0..3 {
                if let Some(value) = stream.next().await {
                    values.push(value);
                }
            }
            assert_eq!(values, vec![1, 2, 3]);
        });
    }

    #[test]
    fn test_combined_streams() {
        block_on(async {
            let result: Vec<String> = combined_streams().take(10).collect().await;

            // Check if we have the correct number of items
            assert_eq!(result.len(), 10);

            // Check if we have items from all three streams
            assert!(result.iter().any(|s| s.starts_with("Count:")));
            assert!(result.iter().any(|s| s.starts_with("Fibonacci:")));

            // Check specific items
            assert!(result.contains(&"Count: 0".to_string()));
            assert!(result.contains(&"Fibonacci: 3".to_string()));
        });
    }

    #[test]
    fn test_poll_next() {
        let instance = create_stream_count_stream();

        block_on(async {
            let mut results = Vec::new();
            for _ in 0..5 {
                if let Some(value) = instance.next().await {
                    results.push(value);
                }
            }

            assert_eq!(results, vec![0, 1, 2, 3, 4]);
        });
    }

    #[test]
    fn test_multiple_streams() {
        block_on(async {
            let instance1 = create_stream_count_stream();
            let instance2 = create_stream_count_stream();

            let result1 = instance1.next().await;
            let result2 = instance2.next().await;

            // Both instances should return the first item (0)
            assert_eq!(result1, Some(0));
            assert_eq!(result2, Some(0));

            // The next call should return the second item (1) for both instances
            let result3 = instance1.next().await;
            let result4 = instance2.next().await;

            assert_eq!(result3, Some(1));
            assert_eq!(result4, Some(1));
        });
    }

    #[test]
    fn test_stream_exhaustion() {
        block_on(async {
            let instance = create_stream_count_stream();

            // Consume all items
            for _ in 0..5 {
                print!("{:?}", instance.next().await);
            }

            // The next call should return None
            assert_eq!(instance.next().await, None);
        });
    }

    // #[tokio::test]
    // async fn test_error_stream() {
    //     let mut stream = error_stream();
    //     let mut results = Vec::new();

    //     while let Some(item) = stream.next().await {
    //         match item {
    //             Ok(value) => results.push(value),
    //             Err(e) => {
    //                 results.push(-1); // Using -1 to indicate an error occurred
    //                 println!("Stream error: {}", e);
    //             }
    //         }
    //     }

    //     assert_eq!(results, vec![1, 2, -1, 4]);
    // }

    // #[tokio::test]
    // async fn test_combined_error_streams() {
    //     let mut stream = combined_error_streams();
    //     let mut results = Vec::new();

    //     while let Some(item) = stream.next().await {
    //         match item {
    //             Ok(value) => results.push(value),
    //             Err(e) => {
    //                 results.push("Error".to_string());
    //                 println!("Combined stream error: {}", e);
    //             }
    //         }
    //     }

    //     assert_eq!(
    //         results,
    //         vec![
    //             "Count: 0".to_string(),
    //             "Count: 1".to_string(),
    //             "Count: 2".to_string(),
    //             "Fibonacci: 0".to_string(),
    //             "Fibonacci: 1".to_string(),
    //             "Error".to_string(),
    //         ]
    //     );
    // }

    // #[tokio::test]
    // async fn test_error_stream_with_timeout() {
    //     let mut stream = error_stream();
    //     let result = timeout(Duration::from_secs(1), async {
    //         let mut collected = Vec::new();
    //         while let Some(item) = stream.next().await {
    //             collected.push(item);
    //         }
    //         collected
    //     })
    //     .await;

    //     match result {
    //         Ok(items) => {
    //             assert_eq!(
    //                 items,
    //                 vec![
    //                     Ok(1),
    //                     Ok(2),
    //                     Err(StreamErrorInt::IntegerError("An error occurred".to_string())),
    //                     Ok(4)
    //                 ]
    //             );
    //         }
    //         Err(_) => panic!("Timeout occurred while collecting error stream"),
    //     }
    // }

    // #[tokio::test]
    // async fn test_combined_error_streams_handling() {
    //     let mut stream = combined_error_streams();
    //     let mut counts = 0;
    //     let mut fibs = 0;
    //     let mut errors = 0;

    //     while let Some(item) = stream.next().await {
    //         let item: Result<String, StreamErrorString> = item; // Explicit type annotation

    //         match item {
    //             Ok(ref s) if s.starts_with("Count:") => counts += 1,
    //             Ok(ref s) if s.starts_with("Fibonacci:") => fibs += 1,
    //             Err(_) => errors += 1,
    //             _ => {}
    //         }
    //     }

    //     assert_eq!(counts, 3);
    //     assert_eq!(fibs, 2); // One Fibonacci stream yields an error
    //     assert_eq!(errors, 1);
    // }
}

uniffi::include_scaffolding!("api");
