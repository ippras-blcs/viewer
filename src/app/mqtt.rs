use anyhow::{Result, bail};
use arrow::{array::RecordBatch, ipc::reader::StreamReader};
use bytes::{Bytes, buf::Buf};
use egui::{Context, Id};
use object_store::{memory::InMemory, path::Path};
use parquet::arrow::{AsyncArrowWriter, async_writer::ParquetObjectWriter};
use rumqttc::{Client, Event, Incoming, MqttOptions, QoS};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, instrument, trace};

pub(crate) const TOPIC: &str = "ippras.ru/blcs/#";
pub(crate) const TOPIC_ATUC: &str = "ippras.ru/blcs/atuc";
pub(crate) const TOPIC_DDOC_C1: &str = "ippras.ru/blcs/ddoc/c1"; // mA
pub(crate) const TOPIC_DDOC_C2: &str = "ippras.ru/blcs/ddoc/c2"; // mA
pub(crate) const TOPIC_DDOC_T1: &str = "ippras.ru/blcs/ddoc/t1"; // °C
pub(crate) const TOPIC_DDOC_T2: &str = "ippras.ru/blcs/ddoc/t2"; // °C
pub(crate) const TOPIC_DDOC_V1: &str = "ippras.ru/blcs/ddoc/v1"; // mg/L
pub(crate) const TOPIC_DDOC_V2: &str = "ippras.ru/blcs/ddoc/v2"; // %
pub(crate) const TOPIC_DTEC: &str = "ippras.ru/blcs/dtec";

const ID: &str = "ippras.ru/blcs/viewer";
const HOST: &str = "broker.emqx.io";
const PORT: u16 = 1883;

#[cfg(target_arch = "wasm32")]
fn spawn(ctx: &egui::Context) {
    let (mut sender, receiver) = loop {
        // broker.emqx.io:8084
        // match ewebsock::connect("wss://broker.emqx.io:8084/mqtt", Default::default()) {
        match ewebsock::connect("wss://echo.websocket.org", Default::default()) {
            Ok((sender, receiver)) => break (sender, receiver),
            Err(error) => error!(%error),
        }
    };
    spawn(async move {
        // sender.send(ewebsock::WsMessage::Text("Hello!".into()));
        loop {
            sender.send(ewebsock::WsMessage::Text("Hello!".into()));
        }
    });
    spawn(async move {
        // sender.send(ewebsock::WsMessage::Text("Hello!".into()));
        while let Some(event) = receiver.try_recv() {
            println!("Received {:?}", event);
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn spawn(context: &Context) {
    let context = context.clone();
    std::thread::spawn(move || futures::executor::block_on(handler(context)).ok());
}

#[instrument(err)]
async fn handler(context: Context) -> Result<()> {
    let mut options = MqttOptions::new(ID, HOST, PORT);
    options.set_keep_alive(Duration::from_secs(9));
    let (client, mut connection) = Client::new(options, 9);
    client.subscribe(TOPIC, QoS::ExactlyOnce)?;
    for event in connection.iter() {
        handle(&context, event?).await.ok();
    }
    Ok(())
}

#[instrument(err)]
async fn handle(context: &Context, event: Event) -> Result<()> {
    if let Event::Incoming(Incoming::Publish(publish)) = event {
        match &*publish.topic {
            TOPIC_DTEC => {
                let batch = read(publish.payload)?;
                trace!("batch: {batch:?}");
                write(context, TOPIC_DTEC, &batch).await?;
            }
            TOPIC_ATUC => {
                let batch = read(publish.payload)?;
                trace!("batch: {batch:?}");
                write(context, TOPIC_ATUC, &batch).await?;
            }
            topic => error!("Unexpected MQTT topic {topic}"),
        }
    }
    Ok(())
}

#[instrument(err)]
fn read(bytes: Bytes) -> Result<RecordBatch> {
    let projection = None; // read all columns
    let reader = StreamReader::try_new(bytes.reader(), projection)?;
    if let Some(batch) = reader.into_iter().next() {
        Ok(batch?)
    } else {
        bail!("No batches found in the stream reader.")
    }
}

#[instrument(err)]
async fn write(context: &Context, path: &str, batch: &RecordBatch) -> Result<()> {
    let store = context.data_mut(|data| {
        data.get_temp_mut_or_insert_with(Id::new(path), || Arc::new(InMemory::new()))
            .clone()
    });
    let writer = ParquetObjectWriter::new(store, Path::from(path));
    let mut writer = AsyncArrowWriter::try_new(writer, batch.schema(), None)?;
    writer.write(batch).await?;
    writer.close().await?;
    Ok(())
}
