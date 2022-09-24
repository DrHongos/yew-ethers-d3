use yew::{function_component, html, Properties};
use ethers::{prelude::*, utils::format_units};

#[derive(Properties, PartialEq)]
pub struct LastBlockProps {
    pub last_block: Block<H256>,
}

#[function_component(LastBlock)]
pub fn table_block(props: &LastBlockProps) -> Html {
    let last_block = &props.last_block;
    html! {
        <table class="table">
            <tr>
                <th>{"key"}</th>
                <th>{"value"}</th>
            </tr>
            <tr>
                <td>{"number"}</td>
                <td>
                    <a href={
                        format!("https://etherscan.io/block/{}", last_block.number.unwrap())
//                                Arc::clone(&self.client.as_ref().unwrap()).block_url(last_block.number.unwrap())
                    } target={"blank"}>
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
}