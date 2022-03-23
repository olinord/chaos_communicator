use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::string::{String};
use std::sync::Arc;

// Message that contains a piece of data
// this data can be anything
pub struct ChaosMessageBuilder {
    data: HashMap<String, (TypeId, Arc<Box<dyn Any>>)>
}

pub struct ChaosMessage {
    data: HashMap<String, (TypeId, Arc<Box<dyn Any>>)>
}

impl ChaosMessageBuilder {
    pub fn new() -> Self {
        return Self {
            data: HashMap::new()
        }
    }

    pub fn with_param<T: Any + Clone + Sync + Send>(mut self, name: &'static str, value: T) -> Self {
        self.data.insert(String::from(name), (TypeId::of::<T>(), Arc::new(Box::new(value))));
        return self;
    }

    pub fn build(self) -> ChaosMessage {
        return ChaosMessage {
            data: self.data
        }
    }
}

impl ChaosMessage {
    pub fn get<T: Any + Clone + Sync + Send>(&self, name: &'static str) -> Option<T> {
        match self.data.get(&String::from(name)) {
            Some((type_id, value)) =>
            {
                assert_eq!(*type_id, TypeId::of::<T>());
                match value.downcast_ref::<T>(){
                    Some(v) => {
                        Some(v.clone())
                    },
                    None => None
                }
            },
            None => None
        }
    }
}

unsafe impl Send for ChaosMessage {}
unsafe impl Sync for ChaosMessage {}



#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn message_can_retieve_parameter_by_name() {
        let message  = ChaosMessageBuilder::new().with_param::<i32>("test", 1123).build();
        assert_eq!(message.get::<i32>("test"), Some(1123));
    }
}

