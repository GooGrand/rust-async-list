use std::sync::Arc;
use tokio::task;
const THERSHOLD: usize = 3;

fn sample(item: u8) -> u8 {
    item*2
}

async fn iterate<T: Copy,R,F>(arc_vector: Arc<Vec<T>>, from: usize, to: usize, func: F) -> Vec<R>  where F: Fn(T) -> R + 'static {
    let mut result = Vec::new();
    println!(" from - {:?}", from);
    println!(" to - {:?}", to);
    for i in arc_vector[from..to].iter() {
        result.push(func(*i));
    }
    result
}


async fn splitter<T: Copy + Sync + Send + 'static, R: Send + Sync + 'static, F: 'static + Send + Sync + Copy>
    (vector: Vec<T>, func: F) -> Vec<R>
where 
    F: Fn(T) -> R
 {
    let length = vector.len();
    let arc_vector = Arc::new(vector);
    let mut handlers = Vec::new();
    let mut from = 0;
    let mut to = from + THERSHOLD; // to count 0 index element
    while from < length {
        if to > length {
            to = length;
        }
        let arc_vector = arc_vector.clone();
        handlers.push(
            task::spawn(         
                async move { 
                    iterate(arc_vector, from, to, func).await
                }
            )
        );
        from = to;
        to += THERSHOLD;
    }
    let mut result = Vec::with_capacity(length);
    println!("Amount of spawned threads {}", handlers.len());
    for i in handlers.into_iter() {
        result.append(&mut i.await.unwrap());
    }
    result
}

#[tokio::main]
async fn main() {
    // run main to see amount spawned threads
    let vector: Vec<u8> = vec![2, 2, 2, 4, 4, 4, 3, 3, 3, 3];
    splitter(vector, sample).await;

    let vector1: Vec<u8> = vec![2, 2, 2];
    splitter(vector1, sample).await;

    let vector2: Vec<u8> = vec![2, 2];
    splitter(vector2, sample).await;
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[tokio::test]
    async fn splitter_test() {
        // test for multithread length
        let vector: Vec<u8> = vec![2, 2, 2, 2, 2, 2, 2, 2, 2, 2];
        let result = splitter(vector, sample).await;
        assert_eq!(result, vec![4, 4, 4, 4, 4, 4, 4, 4, 4, 4]);

        // test for THERSHOLD
        let vector1: Vec<u8> = vec![2, 2, 2];
        let result1 = splitter(vector1, sample).await;
        assert_eq!(result1, vec![4, 4, 4]);

        // test for less than THERSHOULD
        let vector2: Vec<u8> = vec![2, 2];
        let result2 = splitter(vector2, sample).await;
        assert_eq!(result2, vec![4, 4]);

        // test with different value 
        let vector3: Vec<u8> = vec![2, 2, 2, 4, 4, 4, 3, 3, 3, 3];
        let result3 = splitter(vector3, sample).await;
        assert_eq!(result3, vec![4, 4, 4, 8, 8, 8, 6, 6, 6, 6]);

    }
}
