# Causal-path verification

Focused regression result, from the valid D-096 finite-allocation mesh:

| quantity | value |
|---|---:|
| A before | 150.0 |
| A after store | 149.44526627218934 |
| R after store | 0.5547337278106509 |
| A→R | 0.5547337278106509 |
| R→A | 0.010287501371666747 |
| R→W | 0.00027736686390532545 |
| R before growth | 0.5441688595750788 |
| R after growth | 0.5101043491083318 |
| R→M | 0.03406451046674706 |
| R→W from growth | 0.03406451046674706 |
| reserve accounting residual | -9.020562075079e-17 |

The test proves nonzero storage, release, reserve-funded growth, and closed
reserve accounting without changing any D-096 parameters.
