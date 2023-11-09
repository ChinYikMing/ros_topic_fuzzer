use serde::{Deserialize, Serialize};
use serde_json::Value;

extern crate yaml_rust;
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

use arbitrary::{Arbitrary, Result, Unstructured};

use std::{thread, time};

use std::str;

use std::collections::HashMap;

use futures::executor::LocalPool;
use futures::future;
use futures::stream::StreamExt;
use futures::task::LocalSpawnExt;
use r2r::QosProfile;

use r2r::std_msgs::msg::String as R2RString;
use r2r::std_msgs::msg::Int32 as R2RInt32;
use r2r::Publisher;
use r2r::PublisherUntyped;
use r2r::Timer;

#[derive(Copy, Clone, Debug)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl<'a> Arbitrary<'a> for Rgb {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let r = u8::arbitrary(u)?;
        let g = u8::arbitrary(u)?;
        let b = u8::arbitrary(u)?;
        Ok(Rgb { r, g, b })
    }
}

/*
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct R2RString {
    pub data: std::string::String,
}
impl<'a> Arbitrary<'a> for R2RString {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let mut buf = vec![0; u.len()];
        assert!(u.fill_buffer(&mut buf).is_ok());
        let string = String::from_utf8(buf.to_vec()).unwrap();
        Ok(R2RString { data: string })
    }
}
*/

#[derive(Clone, Debug)]
struct R2RInt32Wrapper {
    data: R2RInt32,
}

impl<'a> Arbitrary<'a> for R2RInt32Wrapper {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let x: i32 = u.int_in_range(-1000..=1000).expect("run out of unstructured `u`");
        Ok(R2RInt32Wrapper {
            data: R2RInt32 { data: x },
        })
    }
}

#[derive(Clone, Debug)]
struct R2RStringWrapper {
    data: R2RString,
}

impl<'a> Arbitrary<'a> for R2RStringWrapper {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let mut buf = vec![0; u.len()];
        assert!(u.fill_buffer(&mut buf).is_ok());
        let string = String::from_utf8(buf.to_vec()).unwrap();
        Ok(R2RStringWrapper {
            data: R2RString { data: string },
        })
    }
}

#[derive(Clone, Debug)]
pub struct Test {
    pub str: String,
    pub x: usize,
    pub obj: serde_json::Value,
}

impl<'a> Arbitrary<'a> for Test {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let mut buf = vec![0; u.len()];
        assert!(u.fill_buffer(&mut buf).is_ok());
        let string = String::from_utf8(buf.to_vec()).unwrap();
        Ok(Test {
            str: string,
            x: 23,
            obj: 12.into(),
        })
    }
}

enum R2RMsgDispatcherTable {
    Callback0(Box<dyn Fn(&mut Unstructured) -> Result<Test, arbitrary::Error>>),
    Callback1(Box<dyn Fn(&mut Unstructured) -> Result<R2RStringWrapper, arbitrary::Error>>),
}

//type Funcptr<T> = fn(Unstructured<'a>) -> Result<T>;
type Callback = fn(i32) -> i32;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //fn main() {
    
    // work
    //let mut ros_msg_gen_dispatch_tbl: HashMap<&str, Box<dyn Fn(&mut Unstructured) -> Result<Test, arbitrary::Error>>> = HashMap::new();
    //ros_msg_gen_dispatch_tbl.insert("apple", Box::new(|u| Test::arbitrary(u)));
    
    let mut ros_msg_gen_dispatch_tbl: HashMap<&str, R2RMsgDispatcherTable> = HashMap::new();
    ros_msg_gen_dispatch_tbl.insert("std_msgs/int32", R2RMsgDispatcherTable::Callback0(Box::new(|u| Test::arbitrary(u))));
    ros_msg_gen_dispatch_tbl.insert("std_msgs/String", R2RMsgDispatcherTable::Callback1(Box::new(|u| R2RStringWrapper::arbitrary(u))));

    let yaml_data = r"#
        topics:
            - msg_type: std_msgs/String
              msg_factory: te2
              rate: 5
              qos: 1
              topic_name: stringtopic
    #";

    let docs = YamlLoader::load_from_str(yaml_data).unwrap();
    let doc = &docs[0];
    // println!("{:#?}", doc);

    let topic_vec = doc["topics"].as_vec().unwrap();
    // println!("{:#?}", topic_vec);
    
    /*
    for topic in topic_vec {
        println!("msg_type: {}", topic["msg_type"].as_str().unwrap());
        println!("msg_factory: {}", topic["msg_factory"].as_str().unwrap());
        println!("rate: {}", topic["rate"].as_i64().unwrap());
        println!("qos: {}", topic["qos"].as_i64().unwrap());
        println!("topic_name: {}", topic["topic_name"].as_str().unwrap());
        println!("--------------------------");
    }
    */

   // let mut u = Unstructured::new(&[1, 2, 3, 4]);
    //assert!(!u.is_empty());

    // Generating a `u32` consumes all four bytes of the underlying data, so
    // we become empty afterwards.
    //let _ = u32::arbitrary(&mut u);
    //assert!(u.is_empty());
    //println!("empty");

    //let one_sec = time::Duration::from_secs(1);

    /*
        let mut some_bytes: [u8; 3] = [104, 105, 106];
        loop {
            for byte in some_bytes.iter_mut() {
                *byte += 1;
            }
            /*
            for byte in some_bytes {
                println!("{byte}");
            }
            */
            let mut u = Unstructured::new(&some_bytes);
            let test = R2RStringWrapper::arbitrary(&mut u).unwrap();
            println!("{:#?}", test);

            thread::sleep(one_sec);
        }
    */

    let ctx = r2r::Context::create()?;
    //let subscriber = node.subscribe::<r2r::std_msgs::msg::String>("/topic", QosProfile::default())?;
    let mut node = r2r::Node::create(ctx, "node", "namespace")?;

    let mut pool = LocalPool::new();
    let spawner = pool.spawner();
    let mut dispatcher: &R2RMsgDispatcherTable;

    /*
    subscriber
    spawner.spawn_local(async move {
       subscriber.for_each(|msg| {
             println!("got new msg: {}", msg.data);
             future::ready(())
       }).await
    })?;
    */

    let mut some_bytes: [u8; 3] = [104, 105, 106]; // todo! fuzz data
    let mut u = Unstructured::new(&some_bytes);

    for topic in topic_vec {
        let topic_msg_type = topic["msg_type"].as_str().unwrap();
        let topic_name = topic["topic_name"].as_str().unwrap();
        let mut publisher = node.create_publisher::<R2RString>(topic_name, QosProfile::default())?;
        let mut msg_wrapper: R2RStringWrapper = R2RStringWrapper { data: R2RString { data: String::from("apple") }};
        let mut timer = node.create_wall_timer(std::time::Duration::from_millis(1000))?;

        match topic_msg_type {
            "std_msgs/String"=> {
                publisher = node.create_publisher::<R2RString>(topic_name, QosProfile::default())?;
                msg_wrapper = R2RStringWrapper::arbitrary(&mut u).unwrap();
            },
            "std_msgs/Int32"=> {
                node.create_publisher::<R2RInt32>(topic_name, QosProfile::default())?;
            },
            _ => {
            }
        }

        dispatcher = ros_msg_gen_dispatch_tbl.get(topic_msg_type).unwrap();
        match dispatcher {
            R2RMsgDispatcherTable::Callback0(callback) => {
                let ret = callback(&mut u);
            },
            R2RMsgDispatcherTable::Callback1(callback) => {
                let ret = callback(&mut u);
            }
        }

        spawner.spawn_local(async move {
           let mut counter = 0;
                  loop {
                       let _elapsed = timer.tick().await.unwrap();

                       /*
                       for byte in some_bytes.iter_mut() {
                           *byte += 1;
                       }
                       let mut u = Unstructured::new(&some_bytes);
                       */

                       publisher.publish(&msg_wrapper.data).unwrap();
                       counter += 1;
                  }
           })?;
    }

    loop {
        node.spin_once(std::time::Duration::from_millis(100));
        pool.run_until_stalled();
    }
}
