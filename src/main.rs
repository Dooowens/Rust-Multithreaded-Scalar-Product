extern crate rand;
use rand::Rng;
use std::{
    io,
    thread,
    sync::{
        mpsc :: {self, Receiver, Sender },
        Arc,
        Condvar,
        Mutex
    },
    time::Duration
};

struct PairValues {
    a: i32,
    b: i32
}

impl PairValues {
    pub fn new (a: i32, b: i32) -> PairValues {
        PairValues {
            a: a,
            b: b
        }
    }
}

struct Shared {
    buff : Vec<PairValues>,
    nelem: i32,
    head : usize,
    tail : usize
}

impl Shared {
    
    pub fn new () -> Shared {

        let mut vec : Vec<PairValues> = Vec::with_capacity(5);

        for _ in 0..5 {
            vec.push(PairValues::new(0, 0));
        }

        Shared {
            buff : vec,
            nelem : 0,
            head : 0,
            tail : 0
        }
    }
}

fn main() {
   

    let mut vec_a : Vec<i32> = vec![];
    let mut vec_b : Vec<i32> = vec![];

    let mut size = String::new();
    let mut n_threads = String::new();

    println!("Please input vector size");

    io::stdin()
        .read_line(&mut size)
        .expect("Failed to read line");

    let size: i32 = size.trim().parse().unwrap(); 

    println!("Please input number of threads");
    
    io::stdin()
        .read_line(&mut n_threads)
        .expect("Failed to read line");

    let n_threads: i32 = n_threads.trim().parse().unwrap(); 

    for _ in 0..size {
        vec_a.push(rand::thread_rng().gen_range(1..11));
        vec_b.push(rand::thread_rng().gen_range(1..11));
    }

    println!("Vec_a = {:?}", vec_a);
    println!("Vec_b = {:?}", vec_b);

    let mut handles = vec![];

    let (tx, rx) : (Sender<i32>, Receiver<i32>)  = mpsc::channel();

    let shared = Arc::new((Mutex::new(Shared::new()), Condvar::new(), Condvar::new()));
    
    let shared_prod = Arc::clone(&shared);

    let prod = thread::spawn(move || {
        println!("Produttore creato");

        for i in 0..size+1 {
          

            let (lock, cvarp, cvarc) = &*shared_prod;
            let buff = lock.lock().unwrap();

            if i == size {
                let mut buff = cvarp.wait_while(buff, |buff| buff.nelem != 0).unwrap();
                buff.nelem = -1;
                cvarc.notify_all();
            }

            else {
                let mut buff = cvarp.wait_while(buff, |buff| buff.nelem == 5).unwrap();

                let head = buff.head;
                buff.buff[head].a = vec_a.pop().unwrap();
                buff.buff[head].b = vec_b.pop().unwrap();
                buff.head = (head + 1) % 5;
                buff.nelem += 1;
               /* for j in 0..5 {

                    if buff.buff[j].a == 0 && buff.buff[j].b == 0 {

                        buff.buff[j].a = match vec_a.pop() {
                            Some(x) => x,
                            None => 0
                        };

                        buff.buff[j].b = match vec_b.pop() {
                            Some(x) => x,
                            None => 0
                        };

                        buff.nelem += 1;
                        println!("current elements: ({}, {})", buff.buff[j].a, buff.buff[j].b);
                        break;
                    }
                } */
                cvarc.notify_all();
            }
            
        }
    });

    handles.push(prod);

    for i in 0..n_threads {

        let tx_i = tx.clone();
        let shared_cons = Arc::clone(&shared);

        let cons = thread::spawn(move || {

            println!("Thread n {} created!", i);
            loop {

                let (lock, cvarp, cvarc) = &*shared_cons;
                
                let  buff = lock.lock().unwrap();

                let mut buff = cvarc.wait_while(buff, |buff| buff.nelem == 0).unwrap();

                if buff.nelem == -1 {
                    println!("Thread {} morto", i);
                    break;
                }

                let tail = buff.tail;
                println!(" thread {} nelem {} tail {}", i,buff.nelem, tail);
                tx_i.send(buff.buff[tail].a * buff.buff[tail].b).unwrap();
                buff.tail = (tail + 1) % 5;
                buff.nelem -= 1;

               /* for j in 0..5 {

                    if buff.buff[j].a != 0 || buff.buff[j].b != 0 {
                        tx_i.send(buff.buff[j].a * buff.buff[j].b).unwrap();
                        buff.buff[j].a = 0;
                        buff.buff[j].b = 0;
                        buff.nelem -= 1; 
                        break;
                    }
                } */

                cvarp.notify_all();
                
            };
        });
        handles.push(cons);
    }

    let adder = thread::spawn(move || {
        
        let mut sum :i32 = 0;
        for _ in 0..size {
            sum += rx.recv().unwrap();
        }

        println!("Result is: {}", sum);
    });
    handles.push(adder);

    for h in handles {
        h.join().unwrap();
    }
}
