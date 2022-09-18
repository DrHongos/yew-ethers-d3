/////////////////////////////////////////////////////////////////
//
//      d3
//      research StreamExt and yew agents maybe?        
//
/////////////////////////////////////////////////////////////////
use yew::prelude::*;
use ethers::{prelude::*, utils::format_units};
use std::sync::Arc;
use gloo_timers::callback::Interval;
use wasm_bindgen::prelude::*;
use serde::Serialize;
//use std::time::Duration;

const API_MAINNET_KEY: &str = dotenv!("INFURA_WSS_KEY_MAINNET");

#[wasm_bindgen(module = "/src/jscripts/lineal_chart.js")]
extern "C" {
    #[wasm_bindgen(js_name = "LineChart")]
    #[wasm_bindgen(catch)]
    pub fn LineChart(data: JsValue) -> Result<JsValue, JsValue>;
}

pub enum AppMsg {
    SetClient(Provider<Ws>),
    FetchLastBlock,
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
    list_to_display: Vec<Block<H256>>,
//    streamer: Option<FilterWatcher<Ws, H256>>, // 
// process (https://docs.rs/ethers/0.17.0/ethers/providers/trait.StreamExt.html#)
}
#[derive(Debug, Serialize)]
struct Tick {
    x: u64, // timestamp
    y: String, // whatever
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();
    fn create(ctx: &Context<Self>) -> Self {
//   TEST: STREAM (WS)
        ctx.link().send_future(async {
            match Ws::connect(API_MAINNET_KEY).await {
                Ok(ws) => {
                    let provider = Provider::new(ws);//.interval(Duration::from_millis(12_000));
                    //                     
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
            list_to_display: Vec::new(),
            //streamer: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {            
            AppMsg::FetchLastBlock => {
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
                    Interval::new(12_000, move || link.send_message(AppMsg::FetchLastBlock))
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
                ctx.link().send_message(AppMsg::FetchLastBlock); // first fetch
                ctx.link().send_message(AppMsg::StartInterval); //
                true
            }
            AppMsg::SetLastBlock(bn) => {                
                // moves last_block into list and updates the plot
                if let Some(pb) = self.last_block.clone() {
                    self.list_to_display.push(pb);
                }
                // lets prepare a good struct for the plot
                let display: Vec<Tick> = self
                    .list_to_display
                    .iter()
                    .map(|b| {
                        return Tick {
                            x: b.timestamp.as_u64(), // convert to datetime?
                            y: format_units(b.base_fee_per_gas.unwrap_or_default(), 9).unwrap(),
                        }
                    })
                    .collect();
                let test = serde_wasm_bindgen::to_value(&display).unwrap();
                match LineChart(test) {
                    Ok(_) => {                        
                        ()
                    },
                    Err(err) => {
                        log::error!("Plotting failed! {:?}", err);
                        AppMsg::SetError("Error on plotting".to_string());
                        ()
                    }
                };
                // sets new block (not plotted)
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
                if let Some(error) = &self.error {
                    <p>{ format!("Error: {:?}", error) }</p>
                }

                if !self.client.is_none() {
                    if self.interval.is_none() {
                        <button onclick={ctx.link().callback(|_| AppMsg::StartInterval)}>
                            {"Start interval"}
                        </button>
                    } else {
                        <button onclick={ctx.link().callback(|_| AppMsg::StopInterval)}>
                            <div class={"spinner"}></div>
                            {"Stop interval"}                            
                        </button>                
                    }
                }
                <div id="chart" class="is-fullheight"></div>
                <table>
                    <tr>
                        <th> {"Last block"}</th>
                        <th> {"Previously, on mainnet.."}</th>
                    </tr>
                    <tr>
                    <td>
                        {self.table_block()}
                    </td>
                    <td>
                        {self.prevs_table()}
                    </td>
                    </tr>
                </table>
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
                        <td>
                            <a href={format!("https://etherscan.io/block/{}", last_block.number.unwrap())} target={"blank"} style={"color:white;"}>
                                {last_block.number.unwrap()}
                            </a>
                        </td>
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
    pub fn prevs_table(&self) -> Html {
        let sorted_txs: Vec<Block<H256>> = self.list_to_display
            .clone()
            .into_iter()
            .rev()
            .collect();
        let limited_txs = sorted_txs.iter().take(4);
        log::info!("limited {:?}", limited_txs.len());
        // its 3
        // but then why does it print > 3?
        let prev_txs: Html = limited_txs
            .map(|tx| {
                html!{
                    <tr key={tx.hash.as_ref().unwrap().to_string()}>
                        <td>
                            <a href={format!("https://etherscan.io/block/{}", tx.number.unwrap())} target={"blank"} style={"color:white;"}>
                                {tx.number.unwrap()}
                            </a>
                        </td>
                        <td>{tx.hash.unwrap()}</td>
                        <td>{format!("{:.3} gwei", format_units(tx.base_fee_per_gas.unwrap(), 9).unwrap())}</td>
                        <td>{tx.gas_used}</td>
                        <td>{tx.transactions.len()}</td>
                        <td>{format!("{} bytes", tx.size.unwrap())}</td>
                    </tr>
                }
            }).collect();
        html! {
            <table>
                <tr>
                    <th>{"block number"}</th>
                    <th>{"hash"}</th>
                    <th>{"base fee"}</th>
                    <th>{"gas used"}</th>
                    <th>{"tx's qty"}</th>
                    <th>{"size"}</th>
                </tr>
                { prev_txs }
            </table>
        }
    }
}