# pg_splade (WIP)

pg_splade is a PostgreSQL extension to generate sparse vectors from text data using the SPLADE model. It is designed to work with the pgvector extension for PostgreSQL, which allows for efficient storage and querying of vector data.

## Example

```sql
select encode_document('Currently New York is rainy.', 'mini');  -- return sparsevec of pgvector
----
{512:0.006317289,1005:0.0277019,1030:0.2710025,1047:0.0048320186,1051:0.16862626,1062:0.076174766,2004:0.42219737,2024:0.015105149,2030:0.018756643,2039:0.098384105,2044:0.26576918,2048:0.75105494,2086:0.53563094,2104:0.16770968,2143:0.00615404,2146:0.017886179,2162:0.08895821,2260:1.1012205,2396:0.003611354,2403:0.096713714,2557:0.22985326,2577:0.10482119,2652:0.45359585,2665:0.061623573,2740:0.17343551,2748:0.5355909,2784:0.663264,3468:0.18348758,3476:0.12661059,3523:0.046672992,3613:0.13038012,3729:0.05411671,3804:0.0010477774,3825:0.03484945,4041:0.1769638,4190:0.08994507,4313:0.017822243,4319:0.3372148,4543:1.0578483,4550:0.14584349,4587:0.50340676,4634:1.114457,4786:0.2563195,4861:0.10190259,4903:0.08652108,4955:0.6803383,5405:0.107558094,5473:0.06616022,5698:0.056578703,6397:1.0250852,6614:0.33890104,6746:0.16323672,6843:0.0015262633,6973:0.08566948,7019:0.25084227,7129:0.60462564,7187:0.43315196,7526:0.101620264,8528:0.02099621,8555:0.35065213,8792:0.13151532,8951:0.39289466,9326:0.07197555,9452:0.36599377,9610:0.0931605,9907:0.35901606,10102:0.733655,10362:0.34875667,10525:0.062038314,10790:0.03504699,10840:0.0021500108,10975:0.035176586,11059:0.060942426,11560:0.2762718,11962:0.21265459,12349:0.045124136,12643:0.13892794,12826:0.24661723,13368:0.2607852,13512:0.38897645,13670:0.0965623,14099:0.35853484,14162:0.11131927,14296:0.38988572,14735:0.49620926,14752:0.095678814,14865:0.01042622,15812:0.80413485,16324:0.06763586,16374:1.0653424,16393:0.8350897,16986:0.087049656,18973:0.047959168,19096:0.37969,19184:0.30670184,19194:0.2950823,19840:0.013744945,19940:0.0319295,20655:0.15769438,20982:0.19984324,21690:0.03755288,22089:0.01439414,22275:0.001127918,22483:0.19472802,22722:0.00074883073,24058:0.6578825,24167:0.3102912,24707:0.41533288,24904:0.20803596,24908:0.10389941,25832:0.12763266,26044:0.06845508,26993:0.030840782,27371:0.29048964,27818:0.08360804,27936:0.46650207,28271:0.4944916,29182:0.16675441,29248:0.013609481,29309:0.10629611}/30522

select encode_document('Currently New York is rainy.', 'distill');
----
{1030:0.03045152,1051:0.0983974,2000:0.04653145,2004:0.4259929,2010:0.27806202,2024:0.29956952,2030:0.052609432,2039:0.28120682,2044:0.03561314,2048:0.65749484,2052:0.23975815,2055:0.080154315,2056:0.028321704,2062:0.175705,2067:0.2857144,2086:0.47403067,2096:0.12849604,2104:0.25183693,2105:0.0026292775,2111:0.23828155,2145:0.053293116,2146:0.29846072,2150:0.19055457,2155:0.025948906,2162:0.2489394,2164:0.15304875,2184:0.0470925,2260:0.8668083,2301:0.20292723,2306:0.002536891,2340:0.024679098,2516:0.0673233,2557:0.1317144,2622:0.22757043,2652:0.46431312,2665:0.26690057,2740:0.12540303,2748:0.5386678,2784:0.5857867,2900:0.006541269,3155:0.028590975,3205:0.023408609,3263:0.0019201667,3436:0.024305578,3468:0.0907582,3523:0.20753567,3610:0.2283758,3656:0.19903593,3664:0.32653692,3729:0.0063459557,3786:0.25587493,3893:0.029033769,3916:0.20578693,4041:0.38402838,4149:0.00056536903,4225:0.037782826,4319:0.44483754,4543:0.83871144,4587:0.37997717,4634:0.9721745,4651:0.3350702,4786:0.5629887,4789:0.040951837,4861:0.57092685,4955:0.5790428,5698:0.0013712775,6231:0.20482299,6397:0.8065371,6614:0.31079674,6746:0.21580563,7019:0.18803325,7065:0.06700996,7129:0.5067042,7187:0.417485,7716:0.19527192,7888:0.15358472,8045:0.00048554075,9452:0.3773294,9610:0.16002086,9667:0.035563096,9907:0.014836451,10102:0.6705526,10284:0.08754355,10435:0.07274844,10621:0.24922143,10946:0.025199654,11053:0.18435384,11095:0.017865805,11560:0.09400129,11696:0.06766583,12826:0.17531112,13512:0.5804081,14099:0.29635167,14131:0.046676066,14179:0.26809084,14296:0.19230996,14735:0.38855737,14752:0.06922969,15489:0.009574329,15812:0.53435355,16374:0.7968857,16393:0.6973227,18214:0.37871453,19096:0.41222158,19184:0.14142439,19194:0.023605624,19397:0.040768396,19858:0.03072565,19940:0.31071985,24058:0.5935809,24707:0.52276087,25312:0.18785872,26837:0.004286269,27371:0.12081789,27936:0.42908773,28271:0.081791826,28387:0.030263828,29024:0.107892685}/30522

select encode_query('What''s the weather in ny now?', 'distill');
----
{102:1,103:1,1006:1.5750716,1030:3.3312547,1056:1.4272584,1997:0.13530165,2000:0.49892646,2055:2.7698843,2086:3.5895495,4634:4.5684156,6397:5.7728624}/30522
```

## Install

> You need to install [`pgvector`](https://github.com/pgvector/pgvector) first

1. Build and install the extension.
```sh
cargo pgrx install --release
cp -r assets "$(pg_config --sharedir)/splade"  # copy built-in model, you can ignore this step if you want to download your own model
```

2. Configure your PostgreSQL by modifying the `shared_preload_libraries` to include the extension.
```sh
psql -U postgres -c 'ALTER SYSTEM SET shared_preload_libraries = "pg_splade.so"'
# You need restart the PostgreSQL cluster to take effects.
sudo systemctl restart postgresql.service   # for users running with systemd
```

3. Connect to the database and enable the extension.
```sql
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS pg_splade;
```

## Model

We have a built-in model `distill` which is from `opensearch-project/opensearch-neural-sparse-encoding-doc-v3-distill` on Hugging Face Hub. You can also use other models from Hugging Face Hub by calling `download_model` function. The model will be downloaded and saved in the `splade` directory under the PostgreSQL shared directory. The name of the model is used as the key to access the model in the database.

### Preload

For each connection, postgres will load the model from the disk. If you want to preload the model at the startup, you can set the `splade.preload_models` GUC to a comma-separated list of model names. For example:
```sh
psql -c "ALTER SYSTEM SET splade.preload_models = 'distill'"
sudo systemctl restart postgresql.service   # for users running with systemd
```

## Reference

### Functions

- `encode_document(document text, model text) RETURNS sparsevec` - Encodes a document into a sparse vector using the specified model.
- `encode_query(query text, model text) RETURNS sparsevec` - Encodes a query into a sparse vector using the specified model.
- `truncate_sparsevec(vector sparsevec, chunk int) RETURNS sparsevec` - Truncates a sparse vector to the specified chunk size. It will only keep the top-k elements in the vector. It helps to work with hnsw indexes.
- `download_model(name text, repo_id text)` - Downloads a model from Hugging Face Hub. The model will be saved in the `splade` directory under the PostgreSQL shared directory. The name of the model is used as the key to access the model in the database. The repo_id is the Hugging Face Hub repo ID of the model. For example, `opensearch-project/opensearch-neural-sparse-encoding-doc-v2-mini`.
- `remove_model(name text)` - Removes a model from the `splade` directory.
- `list_model() RETURNS text[]` - Lists all the models in the `splade` directory.

### GUCs

- `splade.preload_models (string)` - A comma-separated list of models to preload. The default is empty.

## Inference Backend

Supported backends:
- `cuda` - NVIDIA GPU with CUDA support.
- `metal` - Apple GPU with Metal support.
- `mkl` - Intel CPU with MKL support.
- `cpu` - CPU with maximum simd support. (default)

If you want to use other backend, you may need to add compile flags when installing the extension. For example, to use `cuda` backend, you can run:
```sh
cargo pgrx install --release --features cuda
```

When enabling multiple backends, it will try using the first one in [`cuda`, `metal`, `mkl`, `cpu`] order.

When using CPU backend (`mkl` or `cpu`), you can change environment variable `RAYON_NUM_THREADS` to control the number of threads used for inference. The default value is the logical CPU count.
