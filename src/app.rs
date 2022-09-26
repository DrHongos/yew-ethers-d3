////////////////////////////////////////////////////////
//          TODO:
//      Input: EOA
    //      Allow fetching data of an account (metamask connected?)
    //      Fetch balance
    //      Fetch & display ERC20/ERC721/ERC1155 transfers
//      Input: Contract address
    //      Read ABI's of contracts (display handler? how to sign?)
    //      Display stats about contracts (as a verifier / anti-spam)
//      Refactor
////////////////////////////////////////////////////////
#![allow(non_snake_case)]
use yew::prelude::*;
use crate::{components::{
//    tracker_table::Tracker,
    erc20_history::ERC20History,
}};
pub enum AppMsg {}
pub struct App {}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();
    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    //fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {}

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <main class="container">
                <ERC20History />
//                <Tracker />
            </main>
        }
    }
    
}
