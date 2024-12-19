module {{module_name}}::{{ module_name }} {
    use sui::coin::{Self, TreasuryCap};
    public struct {{ token_type }} has drop {}

    /// Initialize the token with treasury and metadata
    fun init(witness: {{ token_type }}, ctx: &mut TxContext) {
        let (treasury, metadata) = coin::create_currency(
            witness, {{ decimals }}, b"{{ symbol }}", b"{{ name }}", b"{{ description }}", option::none(), ctx
        );
        {% if is_frozen %}
        transfer::public_freeze_object(metadata);
        {% else %}
        transfer::public_share_object(metadata);
        {% endif %}
        transfer::public_transfer(treasury, ctx.sender());
    }

    public fun mint(
	    treasury_cap: &mut TreasuryCap<{{ token_type }}>,
        amount: u64,
        recipient: address,
        ctx: &mut TxContext,
    ) {
        let coin = coin::mint(treasury_cap, amount, ctx);
        transfer::public_transfer(coin, recipient)
    }
}
