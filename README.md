# electricui-embedded &emsp; ![ci] [![crates.io]](https://crates.io/crates/electricui-embedded) [![docs.rs]](https://docs.rs/electricui-embedded)

An unofficial and incomplete `no_std` Rust library for
implementing the [ElectricUI Binary Protocol][eui-bin-proto].

See the [ElectricUI docs][eui-docs] or the [ElectricUI C library][eui-c-lib] for more information.

## Example

See [electricui-embedded-stm32f4-example](https://github.com/jonlamb-gh/electricui-embedded-stm32f4-example)
for the target portion.

Note: this example is pretty low level and shouldn't be replicated, see the [electricui-cli crate][eui-cli]
for other patterns and examples.

```text
cargo run --example host -- /dev/ttyUSB0

Requesting board ID
>> { DataLen(0), Type(8), Int(1), Offset(0), IdLen(1), Resp(1), Acknum(0) }
<< { DataLen(2), Type(8), Int(1), Offset(0), IdLen(1), Resp(0), Acknum(0) }
Board ID: [EF, BE]
Requesting name
>> { DataLen(0), Type(0), Int(0), Offset(0), IdLen(4), Resp(1), Acknum(0) }
<< { DataLen(8), Type(4), Int(0), Offset(0), IdLen(4), Resp(0), Acknum(0) }
Name: 'my-board'
Requesting writable IDs announcement
>> { DataLen(0), Type(0), Int(1), Offset(0), IdLen(1), Resp(1), Acknum(0) }
<< { DataLen(34), Type(1), Int(1), Offset(0), IdLen(1), Resp(0), Acknum(0) }
<< { DataLen(1), Type(6), Int(1), Offset(0), IdLen(1), Resp(0), Acknum(0) }
Message IDs (4):
  led_blink
  led_state
  lit_time
  name
Got AM_END, count = 4
Requesting tracked variables
>> { DataLen(0), Type(0), Int(1), Offset(0), IdLen(1), Resp(1), Acknum(0) }
<< { DataLen(1), Type(6), Int(0), Offset(0), IdLen(9), Resp(0), Acknum(0) }
Got tracked var Id(led_blink), Type(U8), Data([01])
<< { DataLen(1), Type(6), Int(0), Offset(0), IdLen(9), Resp(0), Acknum(0) }
<< { DataLen(2), Type(8), Int(0), Offset(0), IdLen(8), Resp(0), Acknum(0) }
Got tracked var Id(led_state), Type(U8), Data([01])
Got tracked var Id(lit_time), Type(U16), Data([46, 00])
<< { DataLen(8), Type(4), Int(0), Offset(0), IdLen(4), Resp(0), Acknum(0) }
Got tracked var Id(name), Type(Char), Data([74, 69, 6D, 65, 46, 00, 53, 62])
Requesting heartbeat val=3
>> { DataLen(1), Type(6), Int(1), Offset(0), IdLen(1), Resp(1), Acknum(0) }
<< { DataLen(1), Type(6), Int(1), Offset(0), IdLen(1), Resp(0), Acknum(0) }
Got heartbeat val=3
```

## Protocol Diagram

![protocol](res/protocol.png)

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

[ci]: https://github.com/jonlamb-gh/electricui-embedded-rs/workflows/CI/badge.svg
[crates.io]: https://img.shields.io/crates/v/electricui-embedded.svg
[docs.rs]: https://docs.rs/electricui-embedded/badge.svg
[eui-docs]: https://electricui.com/docs/
[eui-bin-proto]: https://electricui.com/docs/hardware/protocol
[eui-c-lib]: https://github.com/electricui/electricui-embedded
[eui-cli]: https://github.com/jonlamb-gh/electricui-cli
