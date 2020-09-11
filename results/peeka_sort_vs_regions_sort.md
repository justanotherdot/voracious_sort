# Peeka sort vs Regions sort

Computer: Ryzen 9 3950x (16 physical cores, 32 threads), 32GB RAM DDR4, MB X570 TUF Gaming

Peeka sort:
- Language: Rust
- Voracious radix sort v1.0.0
- chunk_size: 1_000_000
- threads: 16
```
Please note that with more than 16 threads given to Rayon threadpool,
performance decreases.
```
```
Peeka sort can sort all types supported by the crate. A dedicated implementation
for unsigned integer should be faster.
```

Regions sort: Clone from the [repository](https://github.com/omarobeya/parallel-inplace-radixsort).
- Language: C++
- threads: 32

## Results
<table style="text-align: right;">
  <thead>
    <tr><td colspan = 4>u32</td><td colspan = 2>u64</td></tr>
    <tr>
      <td>Array size</td><td>Distribution</td>
      <td>Peeka sort (5 runs)</td><td>Regions sort (1 run)</td>
      <td>Peeka sort (5 runs)</td><td>Regions sort (1 run)</td>
    </tr>
  </thead>
  <tbody>
    <tr><td rowspan = 5>1_000_000</td><td>Unif</td><td>7305us</td><td>15ms</td><td>4128us</td><td>15ms</td></tr>
    <tr><td>Unif 10^9</td><td>6910us</td><td>4ms</td><td>3148us</td><td>4ms</td></tr>
    <tr><td>Normal 10</td><td>6311us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 20</td><td>7327us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 30</td><td>7624us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td rowspan = 5>5_000_000</td><td>Unif</td><td>10442us</td><td>17ms</td><td>14524us</td><td>21ms</td></tr>
    <tr><td>Unif 10^9</td><td>14320us</td><td>8ms</td><td>14025us</td><td>9ms</td></tr>
    <tr><td>Normal 10</td><td>9339us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 20</td><td>11605us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 30</td><td>13885us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td rowspan = 5>10_000_000</td><td>Unif</td><td>21356us</td><td>20ms</td><td>26345us</td><td>30ms</td></tr>
    <tr><td>Unif 10^9</td><td>21380us</td><td>15ms</td><td>26210us</td><td>24ms</td></tr>
    <tr><td>Normal 10</td><td>11916us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 20</td><td>14178us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 30</td><td>20434us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td rowspan = 5>20_000_000</td><td>Unif</td><td>42170us</td><td>30ms</td><td>54219us</td><td>45ms</td></tr>
    <tr><td>Unif 10^9</td><td>39730us</td><td>31ms</td><td>50746us</td><td>48ms</td></tr>
    <tr><td>Normal 10</td><td>18673us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 20</td><td>21943us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 30</td><td>29681us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td rowspan = 5>50_000_000</td><td>Unif</td><td>67753us</td><td>63ms</td><td>125245us</td><td>133ms</td></tr>
    <tr><td>Unif 10^9</td><td>66497us</td><td>66ms</td><td>113464us</td><td>123ms</td></tr>
    <tr><td>Normal 10</td><td>47045us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 20</td><td>64507us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 30</td><td>62432us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td rowspan = 5>100_000_000</td><td>Unif</td><td>112052us</td><td>143ms</td><td>201599us</td><td>254ms</td></tr>
    <tr><td>Unif 10^9</td><td>110348us</td><td>135ms</td><td>193891us</td><td>246ms</td></tr>
    <tr><td>Normal 10</td><td>96008us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 20</td><td>117573us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 30</td><td>165002us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td rowspan = 5>200_000_000</td><td>Unif</td><td>275524us</td><td>266ms</td><td>462552us</td><td>514ms</td></tr>
    <tr><td>Unif 10^9</td><td>266960us</td><td>275ms</td><td>448841us</td><td>498ms</td></tr>
    <tr><td>Normal 10</td><td>201092us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 20</td><td>246312us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 30</td><td>277219us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td rowspan = 5>300_000_000</td><td>Unif</td><td>401652us</td><td>402ms</td><td>709877us</td><td>769ms</td></tr>
    <tr><td>Unif 10^9</td><td>387881us</td><td>393ms</td><td>689097us</td><td>749ms</td></tr>
    <tr><td>Normal 10</td><td>313019us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 20</td><td>389341us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 30</td><td>378574us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td rowspan = 5>400_000_000</td><td>Unif</td><td>500181us</td><td>556ms</td><td>983145us</td><td>1013ms</td></tr>
    <tr><td>Unif 10^9</td><td>496936us</td><td>514ms</td><td>974892us</td><td>996ms</td></tr>
    <tr><td>Normal 10</td><td>425905us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 20</td><td>503749us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 30</td><td>565395us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td rowspan = 5>500_000_000</td><td>Unif</td><td>618062us</td><td>711ms</td><td>1254822us</td><td>1278ms</td></tr>
    <tr><td>Unif 10^9</td><td>687115us</td><td>641ms</td><td>1258222us</td><td>1238ms</td></tr>
    <tr><td>Normal 10</td><td>556963us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 20</td><td>648975us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 30</td><td>741408us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td rowspan = 5>600_000_000</td><td>Unif</td><td>809774us</td><td>870ms</td><td>1534043us</td><td>1536ms</td></tr>
    <tr><td>Unif 10^9</td><td>769638us</td><td>766ms</td><td>1505999us</td><td>1479ms</td></tr>
    <tr><td>Normal 10</td><td>668700us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 20</td><td>778692us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 30</td><td>956010us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td rowspan = 5>700_000_000</td><td>Unif</td><td>898337us</td><td>1013ms</td><td>1864484us</td><td>1796ms</td></tr>
    <tr><td>Unif 10^9</td><td>888881us</td><td>893ms</td><td>1843330us</td><td>1722ms</td></tr>
    <tr><td>Normal 10</td><td>781308us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 20</td><td>916488us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 30</td><td>1136253us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td rowspan = 5>800_000_000</td><td>Unif</td><td>1075028us</td><td>1124ms</td><td>2113241us</td><td>2063ms</td></tr>
    <tr><td>Unif 10^9</td><td>1003876us</td><td>1018ms</td><td>2130988us</td><td>1960ms</td></tr>
    <tr><td>Normal 10</td><td>907131us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 20</td><td>1050982us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 30</td><td>1312447us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td rowspan = 5>900_000_000</td><td>Unif</td><td>1172373us</td><td>1290ms</td><td>2474502us</td><td>2325ms</td></tr>
    <tr><td>Unif 10^9</td><td>1159603us</td><td>1141ms</td><td>2404237us</td><td>2192ms</td></tr>
    <tr><td>Normal 10</td><td>1054176us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 20</td><td>1197084us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 30</td><td>1508935us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td rowspan = 5>1_000_000_000</td><td>Unif</td><td>1328050us</td><td>1369ms</td><td>2765527us</td><td>2585ms</td></tr>
    <tr><td>Unif 10^9</td><td>1314220us</td><td>1281ms</td><td>2731763us</td><td>2422ms</td></tr>
    <tr><td>Normal 10</td><td>1138785us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 20</td><td>1352689us</td><td>-</td><td>-</td><td>-</td></tr>
    <tr><td>Normal 30</td><td>1680500us</td><td>-</td><td>-</td><td>-</td></tr>
  </tbody>
</table>
