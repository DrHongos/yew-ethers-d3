/////////////////////////////////////////////////////////////////
//
//      interval -> stream
//
/////////////////////////////////////////////////////////////////
use yew::prelude::*;
use ethers::{prelude::*, utils::format_units};
use std::sync::Arc;
use gloo_timers::callback::Interval;
use std::time::Duration;

const API_MAINNET_KEY: &str = dotenv!("INFURA_WSS_KEY_MAINNET");

pub enum AppMsg {
    SetClient(Provider<Ws>),
    FetchBlocks,
    SetLastBlock(Block<H256>),    
    SetError(String),
    StartInterval,
    StopInterval,
}
pub struct App {
    client: Option<Arc<Provider<Ws>>>,
    last_block: Option<Block<H256>>,    
    interval: Option<Interval>,
    error: Option<String>,
    // list_to_display: Vec<Block<H256>> // make a function to:
// let mut stream = provider.watch_blocks().await?.take(x); // where i can pick x: usize
// process (https://docs.rs/ethers/0.17.0/ethers/providers/trait.StreamExt.html#) and display data 
// table + plot
// show-off
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();
    fn create(ctx: &Context<Self>) -> Self {
//   TEST: STREAM (WS)
        ctx.link().send_future(async {
            match Ws::connect(API_MAINNET_KEY).await {
                Ok(ws) => {
                    let provider = Provider::new(ws).interval(Duration::from_millis(12_000));
                    AppMsg::SetClient(provider)
                },
                Err(err) => AppMsg::SetError(err.to_string())
            } 

        });

//   PROVIDER (WS) DISCRETELY CONNECTED AND SET AS ARC<..>
//        ctx.link().send_future(async {
//            match Provider::<Ws>::connect(API_MAINNET_KEY)
//            .await {
//                Ok(prov) => {
//                    // prov.interval(12_000)
//                    AppMsg::SetClient(prov)
//                },
//                Err(err) => AppMsg::SetError(err.to_string())
//            }
//        });
        Self {
            client: None,
            last_block: None,
            error: None,
            interval: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {            
            AppMsg::FetchBlocks => {
                let client = Arc::clone(&self.client.as_ref().unwrap());
                ctx.link().send_future(async move {
                    match client.get_block(BlockNumber::Latest).await {
                        Ok(block) => {
                            let block = block.unwrap();
                            AppMsg::SetLastBlock(block)
                        },
                        Err(err) => AppMsg::SetError(err.to_string())
                    }
                });
                false
            }
            AppMsg::StartInterval => {
                let handle = {
                    let link = ctx.link().clone();
                    Interval::new(12_000, move || link.send_message(AppMsg::FetchBlocks))
                };
                self.interval = Some(handle);
                log::info!("Started interval");
                true
            }
            AppMsg::StopInterval => {
                //handler.cancel(); // HOW TO DEREFERENCE? IS THAT THE GWEI?
                self.interval = None;                // this works!
                log::info!("Stopped interval");
                true
            }
            AppMsg::SetClient(provider) => {
                self.client = Some(Arc::new(provider));
                ctx.link().send_message(AppMsg::FetchBlocks); // first fetch
                true
            }
            AppMsg::SetLastBlock(bn) => {
                self.last_block = Some(bn);
                true
            }
            AppMsg::SetError(err) => {
                self.error = Some(err);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <main>
                <h1>{ "Hello Rust!" }</h1>
                if !self.client.is_none() {
                    if self.interval.is_none() {
                        <button onclick={ctx.link().callback(|_| AppMsg::StartInterval)}>
                            {"Start interval"}
                        </button>
                    } else {
                        <div>
                            <button onclick={ctx.link().callback(|_| AppMsg::StopInterval)}>
                                <div class={"spinner"}></div>
                                {"Stop interval"}                            
                            </button>                
                        </div>
                    }
                }
                {self.table_block()}
                if let Some(error) = &self.error {
                    <p>{ format!("Error: {:?}", error) }</p>
                }

            </main>
        }
    }
    
}

impl App {
    pub fn table_block(&self) -> Html {
        if let Some(last_block) = &self.last_block {
            html! {
                <table>
                    <tr>
                        <th>{"key"}</th>
                        <th>{"value"}</th>
                    </tr>
                    <tr>
                        <td>{"number"}</td> // make it a link
                        <td>{last_block.number.unwrap()}</td>
                    </tr>
                    <tr>
                        <td>{"hash"}</td>
                        <td>{last_block.hash.unwrap()}</td>
                    </tr>
                    <tr>
                        <td>{"basefee"}</td>
                        <td>
                        {format!("{:.3} gwei", format_units(last_block.base_fee_per_gas.unwrap(), 9).unwrap())}
                        </td>
                    </tr>
                    <tr>
                        <td>{"gas used"}</td>
                        <td>{last_block.gas_used}</td>
                    </tr>
                    <tr>
                        <td>{"tx.len()"}</td>
                        <td>{last_block.transactions.len()}</td>
                    </tr>
                    <tr>
                        <td>{"size"}</td>
                        <td>{last_block.size.unwrap()} {" bytes"}</td>
                    </tr>
                </table>
            }
        } else {
            html! {}
        }
    }
}