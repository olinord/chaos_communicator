use criterion::{black_box, criterion_group, criterion_main, Criterion};
use chaos_communicator::communicator::ChaosCommunicator;
use chaos_communicator::message::ChaosMessageBuilder;

fn message_parameter_lookup(c: &mut Criterion) {
    let message  = ChaosMessageBuilder::new().with_param::<i32>("test", 1123).build();

    c.bench_function("message parameter lookup", |b| b.iter(|| message.get::<i32>(black_box("test")).unwrap()));
}


fn message_parameter_generation(c: &mut Criterion) {
    c.bench_function("message parameter creation",
                     |b| b.iter(||
                                    {
                                        ChaosMessageBuilder::new().with_param::<i32>("test", black_box(1123)).build("test message")
                                    }
    ));
}

fn communicator_send_message(c: &mut Criterion) {

    let mut communicator = ChaosCommunicator::new();
    let _r = communicator.register_for(987654321);
    c.bench_function("communicator sending message",
                     |b| b.iter(||
                         {
                             let message  = ChaosMessageBuilder::new().with_param::<i32>("test", 1123).build(987654321);
                             communicator.send_message( message)
                         }
                     ));
}

criterion_group!(benches, message_parameter_lookup, message_parameter_generation, communicator_send_message);
criterion_main!(benches);