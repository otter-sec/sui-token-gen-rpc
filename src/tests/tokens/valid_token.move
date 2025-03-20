module Mytoken::mytoken {
    use sui::coin::{Self, TreasuryCap};
    public struct MYTOKEN has drop {}

    fun init(witness: MYTOKEN, ctx: &mut TxContext) {
        let (treasury, metadata) = coin::create_currency(
            witness, 8, b"MT", b"My token", b"Tetsing", option::none(), ctx
        );
        
        transfer::public_freeze_object(metadata);
        
        transfer::public_transfer(treasury, ctx.sender());
    }

    public fun mint(
        treasury_cap: &mut TreasuryCap<MYTOKEN>,
        amount: u64,
        recipient: address,
        ctx: &mut TxContext,
    ) {
        let coin = coin::mint(treasury_cap, amount, ctx);
        transfer::public_transfer(coin, recipient)
    }
}
