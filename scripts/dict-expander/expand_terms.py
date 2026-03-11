#!/usr/bin/env python3
"""
IT 용어 확장 스크립트

기존 시드 데이터에 추가 용어를 확장하여 10,000+ 엔트리 목표 달성
"""

from pathlib import Path
from collect_it_terms import ITTerm, ITTermCollector


class ExtendedITTermCollector(ITTermCollector):
    """확장된 IT 용어 수집기."""

    # 추가 시드 데이터
    EXTENDED_SEED_TERMS = {
        "programming_languages": [
            # 추가 프로그래밍 언어
            ("V", "브이", ["Vlang"]),
            ("Zig", "지그", []),
            ("Nim", "님", []),
            ("Crystal", "크리스탈", []),
            ("D", "디", []),
            ("Reason", "리즌", []),
            ("Elm", "엘름", []),
            ("PureScript", "퓨어스크립트", []),
            ("ReScript", "리스크립트", []),
            ("LiveScript", "라이브스크립트", []),
            ("CoffeeScript", "커피스크립트", []),
            ("Racket", "라켓", []),
            ("Scheme", "스킴", []),
            ("Common Lisp", "커먼리스프", []),
            ("Prolog", "프롤로그", []),
            ("Smalltalk", "스몰토크", []),
            ("Tcl", "티씨엘", []),
            ("Bash", "배시", []),
            ("Zsh", "지셸", ["지쉘"]),
            ("PowerShell", "파워셸", ["파워쉘"]),
            ("Visual Basic", "비주얼베이직", ["VB", "비비"]),
            ("Delphi", "델파이", []),
            ("Pascal", "파스칼", []),
            ("Lisp", "리스프", []),
            ("Apex", "에이펙스", []),
            ("Solidity", "솔리디티", []),
            ("Vyper", "바이퍼", []),
            ("Move", "무브", []),
            ("Cairo", "카이로", []),
        ],
        "frameworks_libraries": [
            # 웹 프레임워크 추가
            ("Ruby on Rails", "루비온레일스", ["Rails", "레일스"]),
            ("Laravel", "라라벨", []),
            ("Symfony", "심포니", []),
            ("CodeIgniter", "코드이그나이터", []),
            ("Yii", "이", []),
            ("CakePHP", "케이크PHP", ["케이크피에이치피"]),
            ("Slim", "슬림", []),
            ("Gin", "진", []),
            ("Echo", "에코", []),
            ("Fiber", "파이버", []),
            ("Chi", "치", []),
            ("Actix", "액틱스", []),
            ("Rocket", "로켓", []),
            ("Axum", "액섬", []),
            ("Warp", "워프", []),
            ("Tide", "타이드", []),
            ("Sinatra", "시나트라", []),
            ("Hanami", "하나미", []),
            ("Padrino", "파드리노", []),
            ("Phoenix", "피닉스", []),
            ("Elixir", "엘릭서", []),
            ("Plug", "플러그", []),
            ("Vapor", "베이퍼", []),
            ("Kitura", "키투라", []),
            ("Perfect", "퍼펙트", []),
            ("Ktor", "케이터", []),
            ("Quarkus", "쿼커스", []),
            ("Micronaut", "마이크로너트", []),
            ("Helidon", "헬리돈", []),
            ("Javalin", "자바린", []),
            ("Play Framework", "플레이프레임워크", ["Play", "플레이"]),
            ("Akka", "아카", []),
            ("Vert.x", "버텍스", []),
            ("Ratpack", "랫팩", []),
            ("Dropwizard", "드롭위저드", []),
            ("Spark", "스파크", []),
            ("Blade", "블레이드", []),
            ("Lumen", "루멘", []),
            ("NestJS", "네스트JS", ["네스트제이에스"]),
            ("AdonisJS", "아도니스JS", []),
            ("Fastify", "패스티파이", []),
            ("Hapi", "하피", []),
            ("Koa", "코아", []),
            ("FeathersJS", "페더스JS", []),
            ("SailsJS", "세일스JS", []),
            ("LoopBack", "루프백", []),
            ("Meteor", "미티어", []),

            # 프론트엔드 라이브러리 추가
            ("Preact", "프리액트", []),
            ("Inferno", "인페르노", []),
            ("Lit", "릿", []),
            ("Alpine.js", "알파인", []),
            ("HTMX", "에이치티엠엑스", []),
            ("Stimulus", "스티뮬러스", []),
            ("Marko", "마르코", []),
            ("Mithril", "미스릴", []),
            ("Ember", "엠버", []),
            ("Backbone", "백본", []),
            ("Knockout", "녹아웃", []),
            ("Aurelia", "오렐리아", []),
            ("Polymer", "폴리머", []),
            ("Stencil", "스텐실", []),
            ("Qwik", "퀵", []),

            # 모바일 프레임워크
            ("React Native", "리액트네이티브", []),
            ("Flutter", "플러터", []),
            ("Ionic", "아이오닉", []),
            ("Capacitor", "커패시터", []),
            ("Cordova", "코르도바", []),
            ("Xamarin", "자마린", []),
            ("NativeScript", "네이티브스크립트", []),
            ("Expo", "엑스포", []),

            # 데스크톱 프레임워크
            ("Electron", "일렉트론", []),
            ("Tauri", "타우리", []),
            ("NW.js", "엔더블유제이에스", []),
            ("Neutralino", "뉴트럴리노", []),
            ("Qt", "큐티", []),
            ("GTK", "지티케이", []),
            ("wxWidgets", "더블유엑스위젯", []),

            # ML/AI 라이브러리 추가
            ("JAX", "잭스", []),
            ("Flax", "플랙스", []),
            ("Optax", "옵탁스", []),
            ("MXNet", "엠엑스넷", []),
            ("Caffe", "카페", []),
            ("Theano", "테아노", []),
            ("Chainer", "체이너", []),
            ("PaddlePaddle", "패들패들", []),
            ("ONNX", "오닉스", []),
            ("TensorRT", "텐서알티", []),
            ("OpenVINO", "오픈비노", []),
            ("CoreML", "코어엠엘", []),
            ("ML.NET", "엠엘닷넷", []),
            ("Accord.NET", "어코드닷넷", []),
            ("MLflow", "엠엘플로우", []),
            ("Kubeflow", "큐브플로우", []),
            ("Ray", "레이", []),
            ("Horovod", "호로보드", []),
            ("DeepSpeed", "딥스피드", []),
            ("Megatron", "메가트론", []),
            ("Transformers", "트랜스포머스", []),
            ("Diffusers", "디퓨저스", []),
            ("Accelerate", "액셀러레이트", []),
            ("PEFT", "페프트", []),
            ("bitsandbytes", "비츠앤바이츠", []),
            ("Sentence-Transformers", "센텐스트랜스포머스", []),
            ("spaCy", "스페이씨", []),
            ("NLTK", "엔엘티케이", []),
            ("Gensim", "젠심", []),
            ("AllenNLP", "앨런엔엘피", []),
            ("Flair", "플레어", []),
            ("StanfordNLP", "스탠포드엔엘피", []),

            # 데이터 처리
            ("NumPy", "넘파이", []),
            ("Pandas", "판다스", []),
            ("Polars", "폴라스", []),
            ("Dask", "다스크", []),
            ("Vaex", "백스", []),
            ("Modin", "모딘", []),
            ("Datatable", "데이터테이블", []),
            ("Apache Arrow", "아파치애로우", ["Arrow", "애로우"]),
            ("Parquet", "파케이", []),
            ("Avro", "아브로", []),
            ("Thrift", "쓰리프트", []),
            ("Cap'n Proto", "캡앤프로토", []),
            ("FlatBuffers", "플랫버퍼스", []),

            # 데이터 시각화
            ("Matplotlib", "맷플롯립", []),
            ("Seaborn", "시본", []),
            ("Plotly", "플롯리", []),
            ("Bokeh", "보케", []),
            ("Altair", "알테어", []),
            ("Holoviews", "홀로뷰스", []),
            ("Dash", "대시", []),
            ("Streamlit", "스트림릿", []),
            ("Gradio", "그라디오", []),
            ("Voila", "보일라", []),
            ("Panel", "패널", []),
            ("D3.js", "디쓰리", []),
            ("Chart.js", "차트제이에스", []),
            ("ECharts", "이차트", []),
            ("Highcharts", "하이차트", []),
            ("ApexCharts", "에이펙스차트", []),
            ("Recharts", "리차트", []),
            ("Victory", "빅토리", []),
            ("Nivo", "니보", []),
            ("Visx", "비스엑스", []),

            # 테스팅 라이브러리 추가
            ("Vitest", "비테스트", []),
            ("Testing Library", "테스팅라이브러리", []),
            ("Puppeteer", "퍼핏티어", []),
            ("WebdriverIO", "웹드라이버아이오", []),
            ("TestCafe", "테스트카페", []),
            ("Cucumber", "큐컴버", []),
            ("JUnit", "제이유닛", []),
            ("TestNG", "테스트엔지", []),
            ("Mockito", "모키토", []),
            ("WireMock", "와이어목", []),
            ("unittest", "유닛테스트", []),
            ("nose", "노즈", []),
            ("doctest", "독테스트", []),
            ("Hypothesis", "하이포시스", []),
            ("RSpec", "알스펙", []),
            ("Minitest", "미니테스트", []),
            ("Capybara", "카피바라", []),
            ("Factory Bot", "팩토리봇", []),
            ("Faker", "페이커", []),
            ("QuickCheck", "퀵체크", []),

            # 빌드/번들러 추가
            ("Turbopack", "터보팩", []),
            ("SWC", "에스더블유씨", []),
            ("Rome", "로마", []),
            ("Biome", "바이옴", []),
            ("Snowpack", "스노우팩", []),
            ("wmr", "더블유엠알", []),
            ("Brunch", "브런치", []),
            ("FuseBox", "퓨즈박스", []),
            ("Browserify", "브라우저파이", []),
            ("RequireJS", "리콰이어제이에스", []),
            ("SystemJS", "시스템제이에스", []),
        ],
        "cloud_infrastructure": [
            # 클라우드 서비스 추가
            ("DigitalOcean", "디지털오션", []),
            ("Linode", "리노드", []),
            ("Vultr", "볼처", []),
            ("Heroku", "헤로쿠", []),
            ("Vercel", "버셀", []),
            ("Netlify", "넷리파이", []),
            ("Cloudflare", "클라우드플레어", []),
            ("Fastly", "패스틀리", []),
            ("Akamai", "아카마이", []),
            ("Render", "렌더", []),
            ("Railway", "레일웨이", []),
            ("Fly.io", "플라이아이오", []),
            ("Supabase", "수파베이스", []),
            ("PlanetScale", "플래닛스케일", []),
            ("Neon", "네온", []),
            ("CockroachDB", "코크로치DB", ["코크로치디비"]),
            ("TiDB", "티아이DB", []),
            ("YugabyteDB", "유가바이트DB", []),

            # 컨테이너/오케스트레이션 추가
            ("Rancher", "랜처", []),
            ("OpenShift", "오픈시프트", []),
            ("ECS", "이씨에스", ["Elastic Container Service"]),
            ("EKS", "이케이에스", ["Elastic Kubernetes Service"]),
            ("GKE", "지케이이", ["Google Kubernetes Engine"]),
            ("AKS", "에이케이에스", ["Azure Kubernetes Service"]),
            ("Fargate", "파게이트", []),
            ("Cloud Run", "클라우드런", []),
            ("Cloud Functions", "클라우드펑션스", []),
            ("Lambda", "람다", []),
            ("Azure Functions", "애저펑션스", []),
            ("Knative", "네이티브", []),
            ("OpenFaaS", "오픈파스", []),
            ("Fission", "피션", []),
            ("Kubeless", "큐브리스", []),

            # CI/CD 추가
            ("Drone", "드론", []),
            ("Buildkite", "빌드카이트", []),
            ("Bamboo", "밤부", []),
            ("TeamCity", "팀시티", []),
            ("Concourse", "컨코스", []),
            ("Tekton", "텍톤", []),
            ("Spinnaker", "스피네이커", []),
            ("Harness", "하니스", []),
            ("Flux", "플럭스", []),
            ("Argo Workflows", "아르고워크플로우", []),
            ("Argo Rollouts", "아르고롤아웃", []),

            # 모니터링/로깅 추가
            ("Splunk", "스플렁크", []),
            ("Sumo Logic", "수모로직", []),
            ("Dynatrace", "다이나트레이스", []),
            ("AppDynamics", "앱다이나믹스", []),
            ("Sentry", "센트리", []),
            ("Rollbar", "롤바", []),
            ("Bugsnag", "버그스냅", []),
            ("Honeycomb", "허니컴", []),
            ("Lightstep", "라이트스텝", []),
            ("Zipkin", "집킨", []),
            ("OpenTelemetry", "오픈텔레메트리", []),
            ("Vector", "벡터", []),
            ("Fluentd", "플루엔티디", []),
            ("Fluent Bit", "플루엔트비트", []),
            ("Logstash", "로그스태시", []),
            ("Loki", "로키", []),
            ("Tempo", "템포", []),
            ("Mimir", "미미르", []),
            ("Cortex", "코텍스", []),
            ("Thanos", "타노스", []),
            ("VictoriaMetrics", "빅토리아메트릭스", []),
            ("InfluxDB", "인플럭스DB", []),
            ("TimescaleDB", "타임스케일DB", []),
            ("Graphite", "그라파이트", []),
            ("Nagios", "나기오스", []),
            ("Zabbix", "자빅스", []),
            ("Icinga", "이싱가", []),
            ("Sensu", "센수", []),

            # 서비스 메시/API 게이트웨이
            ("Kong", "콩", []),
            ("Traefik", "트래픽", []),
            ("Envoy", "엔보이", []),
            ("NGINX", "엔진엑스", []),
            ("HAProxy", "에이치에이프록시", []),
            ("Caddy", "캐디", []),
            ("Apache", "아파치", []),
            ("Nginx Ingress", "엔진엑스인그레스", []),
            ("Ambassador", "앰배서더", []),
            ("Gloo", "글루", []),
            ("Contour", "콘투어", []),

            # 메시징/이벤트 스트리밍 추가
            ("ActiveMQ", "액티브엠큐", []),
            ("ZeroMQ", "제로엠큐", []),
            ("NSQ", "엔에스큐", []),
            ("Amazon SQS", "아마존에스큐에스", ["SQS"]),
            ("Amazon SNS", "아마존에스엔에스", ["SNS"]),
            ("Google Pub/Sub", "구글펍섭", []),
            ("Azure Service Bus", "애저서비스버스", []),
            ("EventBridge", "이벤트브릿지", []),
            ("Kinesis", "키네시스", []),
            ("Apache Flink", "아파치플링크", ["Flink", "플링크"]),
            ("Apache Beam", "아파치빔", ["Beam", "빔"]),
            ("Apache Storm", "아파치스톰", ["Storm", "스톰"]),
            ("Apache Samza", "아파치삼자", ["Samza", "삼자"]),

            # 스토리지/데이터베이스
            ("Amazon S3", "아마존에스쓰리", ["S3", "에스쓰리"]),
            ("Google Cloud Storage", "구글클라우드스토리지", ["GCS", "지씨에스"]),
            ("Azure Blob Storage", "애저블롭스토리지", []),
            ("Backblaze", "백블레이즈", []),
            ("Wasabi", "와사비", []),
            ("Cloudinary", "클라우디너리", []),
            ("imgix", "이미직스", []),
        ],
        "ai_ml": [
            # LLM 모델
            ("Claude", "클로드", []),
            ("Llama", "라마", ["LLaMA"]),
            ("Mistral", "미스트랄", []),
            ("Gemini", "제미니", []),
            ("PaLM", "팜", []),
            ("Falcon", "팔콘", []),
            ("MPT", "엠피티", []),
            ("Pythia", "피시아", []),
            ("StableLM", "스테이블엘엠", []),
            ("Vicuna", "비쿠나", []),
            ("Alpaca", "알파카", []),
            ("Dolly", "돌리", []),
            ("Bloom", "블룸", []),
            ("OPT", "옵트", []),
            ("T5", "티파이브", []),
            ("UL2", "유엘투", []),
            ("Flan", "플랜", []),

            # 비전 모델
            ("CLIP", "클립", []),
            ("DALL-E", "달리", ["달이"]),
            ("Midjourney", "미드저니", []),
            ("Imagen", "이마젠", []),
            ("Parti", "파티", []),
            ("ControlNet", "컨트롤넷", []),
            ("IP-Adapter", "아이피어댑터", []),
            ("SAM", "샘", ["Segment Anything Model"]),
            ("YOLO", "욜로", []),
            ("Mask R-CNN", "마스크알씨엔엔", []),
            ("EfficientNet", "이피션트넷", []),
            ("ResNet", "레즈넷", []),
            ("VGG", "브이지지", []),
            ("InceptionNet", "인셉션넷", []),
            ("MobileNet", "모바일넷", []),
            ("DenseNet", "덴스넷", []),
            ("Vision Transformer", "비전트랜스포머", ["ViT", "비티"]),

            # 오디오 모델
            ("Whisper", "위스퍼", []),
            ("Wav2Vec", "웨이브투벡", []),
            ("HuBERT", "휴버트", []),
            ("WavLM", "웨이브엘엠", []),
            ("Bark", "바크", []),
            ("MusicGen", "뮤직젠", []),
            ("AudioCraft", "오디오크래프트", []),

            # RL 알고리즘
            ("PPO", "피피오", ["Proximal Policy Optimization"]),
            ("DQN", "디큐엔", ["Deep Q-Network"]),
            ("A3C", "에이쓰리씨", []),
            ("SAC", "샥", ["Soft Actor-Critic"]),
            ("TD3", "티디쓰리", ["Twin Delayed DDPG"]),
            ("DDPG", "디디피지", []),
            ("TRPO", "티알피오", []),
            ("AlphaGo", "알파고", []),
            ("MuZero", "뮤제로", []),
            ("AlphaZero", "알파제로", []),

            # ML 기법 추가
            ("Ensemble", "앙상블", []),
            ("Bagging", "배깅", []),
            ("Boosting", "부스팅", []),
            ("Random Forest", "랜덤포레스트", []),
            ("XGBoost", "엑스지부스트", []),
            ("LightGBM", "라이트지비엠", []),
            ("CatBoost", "캣부스트", []),
            ("AdaBoost", "에이다부스트", []),
            ("Gradient Boosting", "그래디언트부스팅", []),
            ("Decision Tree", "결정트리", ["디시전트리"]),
            ("SVM", "에스브이엠", ["Support Vector Machine", "서포트벡터머신"]),
            ("KNN", "케이엔엔", ["K-Nearest Neighbors"]),
            ("Naive Bayes", "나이브베이즈", []),
            ("Logistic Regression", "로지스틱회귀", []),
            ("Linear Regression", "선형회귀", ["리니어리그레션"]),
            ("Ridge", "릿지", []),
            ("Lasso", "라쏘", []),
            ("ElasticNet", "일래스틱넷", []),
            ("PCA", "피씨에이", ["Principal Component Analysis", "주성분분석"]),
            ("t-SNE", "티에스엔이", []),
            ("UMAP", "유맵", []),
            ("K-Means", "케이민즈", []),
            ("DBSCAN", "디비스캔", []),
            ("Hierarchical Clustering", "계층적군집화", []),

            # 최적화 알고리즘
            ("Adam", "아담", []),
            ("SGD", "에스지디", ["Stochastic Gradient Descent"]),
            ("RMSprop", "알엠에스프롭", []),
            ("AdaGrad", "아다그래드", []),
            ("AdaDelta", "아다델타", []),
            ("Nadam", "나담", []),
            ("RAdam", "알아담", []),
            ("Lookahead", "룩어헤드", []),
            ("LAMB", "램", []),
            ("AdamW", "아담더블유", []),

            # 정규화 기법
            ("Layer Normalization", "레이어정규화", []),
            ("Group Normalization", "그룹정규화", []),
            ("Instance Normalization", "인스턴스정규화", []),
            ("Weight Decay", "가중치감쇠", []),
            ("Early Stopping", "얼리스탑핑", []),
            ("Data Augmentation", "데이터증강", []),
            ("Label Smoothing", "레이블스무딩", []),
            ("Mixup", "믹스업", []),
            ("CutMix", "컷믹스", []),
            ("AutoAugment", "오토어그먼트", []),

            # NLP 개념 추가
            ("BLEU Score", "블루스코어", []),
            ("Perplexity", "퍼플렉시티", ["혼란도"]),
            ("Cosine Similarity", "코사인유사도", []),
            ("Semantic Search", "시맨틱서치", ["의미검색"]),
            ("Vector Database", "벡터데이터베이스", []),
            ("ChromaDB", "크로마DB", []),
            ("Pinecone", "파인콘", []),
            ("Weaviate", "위비에이트", []),
            ("Qdrant", "큐드런트", []),
            ("Milvus", "밀버스", []),
            ("FAISS", "페이스", []),
            ("Annoy", "어노이", []),
        ],
        "general_it": [
            # 데이터베이스 추가
            ("Firebase", "파이어베이스", []),
            ("Supabase", "수파베이스", []),
            ("Firestore", "파이어스토어", []),
            ("Realtime Database", "리얼타임데이터베이스", []),
            ("ArangoDB", "아랑고DB", []),
            ("OrientDB", "오리엔트DB", []),
            ("RavenDB", "레이븐DB", []),
            ("CouchDB", "카우치DB", []),
            ("RethinkDB", "리씽크DB", []),
            ("ScyllaDB", "실라DB", []),
            ("ClickHouse", "클릭하우스", []),
            ("Druid", "드루이드", []),
            ("Presto", "프레스토", []),
            ("Trino", "트리노", []),
            ("Impala", "임팔라", []),
            ("Hive", "하이브", []),
            ("HBase", "에이치베이스", []),
            ("Accumulo", "어큐뮬로", []),
            ("FaunaDB", "파우나DB", []),
            ("EdgeDB", "엣지DB", []),
            ("SurrealDB", "서리얼DB", []),

            # 캐시/인메모리
            ("Memcached", "멤캐시디", []),
            ("Hazelcast", "헤이즐캐스트", []),
            ("Ehcache", "이에이치캐시", []),
            ("Ignite", "이그나이트", []),
            ("Coherence", "코히어런스", []),
            ("Aerospike", "에어로스파이크", []),
            ("Dragonfly", "드래곤플라이", []),
            ("KeyDB", "키DB", []),
            ("Valkey", "밸키", []),

            # 검색 엔진
            ("Solr", "솔라", []),
            ("Lucene", "루씬", []),
            ("Algolia", "알골리아", []),
            ("MeiliSearch", "메일리서치", []),
            ("Typesense", "타입센스", []),
            ("Sonic", "소닉", []),
            ("Blast", "블라스트", []),
            ("Manticore", "맨티코어", []),
            ("Zinc", "징크", []),

            # IDE/에디터 추가
            ("WebStorm", "웹스톰", []),
            ("PhpStorm", "피에이치피스톰", []),
            ("RubyMine", "루비마인", []),
            ("GoLand", "고랜드", []),
            ("CLion", "씨라이온", []),
            ("DataGrip", "데이터그립", []),
            ("Rider", "라이더", []),
            ("Fleet", "플리트", []),
            ("Cursor", "커서", []),
            ("Zed", "제드", []),
            ("Nova", "노바", []),
            ("Atom", "아톰", []),
            ("Brackets", "브래킷", []),
            ("CodeSandbox", "코드샌드박스", []),
            ("StackBlitz", "스택블리츠", []),
            ("Replit", "레플릿", []),
            ("Gitpod", "깃팟", []),
            ("Codespaces", "코드스페이스", []),

            # VCS/협업
            ("Bitbucket", "비트버킷", []),
            ("Gitea", "기티아", []),
            ("Gogs", "고그스", []),
            ("Forgejo", "포르제요", []),
            ("SourceForge", "소스포지", []),
            ("Mercurial", "머큐리얼", []),
            ("SVN", "에스브이엔", ["Subversion", "서브버전"]),
            ("Perforce", "퍼포스", []),
            ("CVS", "씨브이에스", []),

            # 프로젝트 관리
            ("Jira", "지라", []),
            ("Confluence", "컨플루언스", []),
            ("Trello", "트렐로", []),
            ("Asana", "아사나", []),
            ("Monday", "먼데이", []),
            ("ClickUp", "클릭업", []),
            ("Linear", "리니어", []),
            ("Notion", "노션", []),
            ("Obsidian", "옵시디언", []),
            ("Roam", "로암", []),
            ("Logseq", "로그시큐", []),

            # 디자인/프로토타이핑
            ("Figma", "피그마", []),
            ("Sketch", "스케치", []),
            ("Adobe XD", "어도비엑스디", []),
            ("InVision", "인비전", []),
            ("Framer", "프레이머", []),
            ("Axure", "액슈어", []),
            ("Balsamiq", "발사믹", []),
            ("Penpot", "펜팟", []),

            # API 개발/테스팅
            ("Postman", "포스트맨", []),
            ("Insomnia", "인섬니아", []),
            ("HTTPie", "에이치티티피", []),
            ("curl", "컬", []),
            ("Swagger", "스웨거", []),
            ("OpenAPI", "오픈API", ["오픈에이피아이"]),
            ("Stoplight", "스탑라이트", []),
            ("Paw", "포", []),
            ("RapidAPI", "래피드API", []),

            # 모니터링/분석
            ("Google Analytics", "구글애널리틱스", []),
            ("Mixpanel", "믹스패널", []),
            ("Amplitude", "앰플리튜드", []),
            ("Segment", "세그먼트", []),
            ("Heap", "힙", []),
            ("Hotjar", "핫자", []),
            ("FullStory", "풀스토리", []),
            ("LogRocket", "로그로켓", []),
            ("Plausible", "플로저블", []),
            ("Umami", "우마미", []),
            ("Fathom", "패덤", []),

            # 보안/인증
            ("Auth0", "오쓰제로", []),
            ("Okta", "옥타", []),
            ("Keycloak", "키클록", []),
            ("Clerk", "클러크", []),
            ("Supertokens", "슈퍼토큰스", []),
            ("NextAuth", "넥스트오쓰", []),
            ("Passport", "패스포트", []),
            ("1Password", "원패스워드", []),
            ("Bitwarden", "비트워든", []),
            ("LastPass", "라스트패스", []),
            ("Vault", "볼트", []),
            ("SOPS", "솝스", []),
            ("Sealed Secrets", "실드시크릿", []),
        ],
    }

    def collect_extended_terms(self) -> None:
        """확장 용어 수집."""
        print("Collecting extended terms...")

        for category, terms in self.EXTENDED_SEED_TERMS.items():
            for term_data in terms:
                if len(term_data) == 3:
                    surface, reading, variants = term_data
                else:
                    surface, reading = term_data
                    variants = []

                # 메인 용어 추가
                it_term = ITTerm(
                    surface=surface,
                    category=category,
                    reading=reading,
                    variants=variants,
                )
                self.terms[category].add(it_term)

                # 한글 용어 추가
                if reading and reading != surface:
                    hangul_term = ITTerm(
                        surface=reading,
                        category=category,
                        reading=reading,
                        variants=[],
                    )
                    self.terms[category].add(hangul_term)

                # 변이형 추가
                for variant in variants:
                    if variant and variant != surface:
                        variant_term = ITTerm(
                            surface=variant,
                            category=category,
                            reading=reading,
                            variants=[],
                        )
                        self.terms[category].add(variant_term)

        # 통계 출력
        total = sum(len(terms) for terms in self.terms.values())
        print(f"Collected {total} extended terms")


def main() -> None:
    """메인 함수."""
    import argparse

    parser = argparse.ArgumentParser(
        description="IT 용어를 확장하여 10,000+ 엔트리 생성"
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path("/home/mare/mecab-ko/data/domain-dic"),
        help="출력 디렉토리 경로",
    )

    args = parser.parse_args()

    collector = ExtendedITTermCollector(args.output_dir)

    # 1. 기본 시드 용어 수집
    collector.collect_seed_terms()

    # 2. 확장 용어 수집
    collector.collect_extended_terms()

    # 3. 복합어 생성
    collector.generate_compound_terms()

    # 4. MeCab CSV로 내보내기
    exported_files = collector.export_to_mecab_csv()

    # 5. 통계 생성 및 저장
    stats = collector.generate_statistics()
    collector.save_statistics(stats)

    # 6. 요약 출력
    print("\n" + "="*60)
    print("Extended IT Term Collection Summary")
    print("="*60)
    print(f"Total terms: {stats['total_terms']}")
    print("\nBy category:")
    for cat, cat_stats in stats["categories"].items():
        print(f"  {cat}: {cat_stats['count']} terms")
    print("\nExported files:")
    for cat, file_path in exported_files.items():
        print(f"  {file_path}")
    print("="*60)


if __name__ == "__main__":
    main()
