# Python 바인딩

MeCab-Ko는 PyO3를 사용하여 Python 바인딩을 제공합니다.

## 설치

```bash
pip install mecab-ko
```

소스에서 설치:

```bash
cd python
pip install maturin
maturin develop --release
```

## 빠른 시작

```python
from mecab_ko import Tagger

# Tagger 생성
tagger = Tagger()

# 문장 분석
result = tagger.parse("안녕하세요")
print(result)

# 노드 단위 처리
for node in tagger.parse_nodes("형태소 분석"):
    print(f"{node.surface}\t{node.pos}\t{node.feature}")
```

## API 레퍼런스

### `Tagger` 클래스

형태소 분석기의 메인 클래스입니다.

```python
class Tagger:
    """MeCab-Ko 형태소 분석기"""

    def __init__(
        self,
        dict_dir: str | None = None,
        user_dict: str | None = None,
        output_format: str = "mecab",
        space_penalty: int = -1000,
    ):
        """
        새로운 Tagger를 생성합니다.

        Args:
            dict_dir: 사전 디렉토리 경로
            user_dict: 사용자 사전 파일 경로
            output_format: 출력 포맷 ("mecab", "wakati", "json", "csv")
            space_penalty: 띄어쓰기 패널티 (기본값: -1000)

        Raises:
            RuntimeError: 사전을 찾을 수 없거나 초기화 실패 시
        """
        ...

    def parse(self, text: str) -> str:
        """
        텍스트를 분석하고 문자열로 반환합니다.

        Args:
            text: 분석할 텍스트

        Returns:
            분석 결과 문자열

        Raises:
            RuntimeError: 파싱 실패 시
        """
        ...

    def parse_nodes(self, text: str) -> list[Node]:
        """
        텍스트를 분석하고 Node 리스트로 반환합니다.

        Args:
            text: 분석할 텍스트

        Returns:
            Node 객체 리스트

        Raises:
            RuntimeError: 파싱 실패 시
        """
        ...

    def parse_to_dict(self, text: str) -> dict:
        """
        텍스트를 분석하고 딕셔너리로 반환합니다.

        Args:
            text: 분석할 텍스트

        Returns:
            {"text": str, "nodes": [{"surface": str, "feature": str, ...}]}

        Raises:
            RuntimeError: 파싱 실패 시
        """
        ...

    def parse_nbest(self, text: str, n: int = 3) -> list[str]:
        """
        N-best 결과를 반환합니다.

        Args:
            text: 분석할 텍스트
            n: 반환할 후보 개수

        Returns:
            N개의 분석 결과 문자열 리스트

        Raises:
            RuntimeError: 파싱 실패 시
        """
        ...
```

### `Node` 클래스

분석된 형태소 노드를 나타내는 클래스입니다.

```python
class Node:
    """형태소 분석 노드"""

    @property
    def surface(self) -> str:
        """표면형 (실제 텍스트)"""
        ...

    @property
    def feature(self) -> str:
        """품사 및 의미 정보"""
        ...

    @property
    def pos(self) -> str:
        """품사 태그"""
        ...

    @property
    def start(self) -> int:
        """시작 위치 (바이트 단위)"""
        ...

    @property
    def length(self) -> int:
        """길이 (바이트 단위)"""
        ...

    @property
    def cost(self) -> int:
        """비용"""
        ...

    @property
    def base_form(self) -> str | None:
        """기본형 (원형)"""
        ...

    @property
    def reading(self) -> str | None:
        """읽기 정보"""
        ...

    def to_dict(self) -> dict:
        """딕셔너리로 변환"""
        ...
```

## 사용 예제

### 기본 사용

```python
from mecab_ko import Tagger

tagger = Tagger()

# 텍스트 분석
text = "아버지가방에들어가신다"
result = tagger.parse(text)
print(result)
```

출력:
```
아버지    NNG,*,F,아버지,*,*,*,*
가        JKS,*,F,가,*,*,*,*
방        NNG,*,T,방,*,*,*,*
에        JKB,*,F,에,*,*,*,*
들어가    VV,*,F,들어가,*,*,*,*
시        EP,*,F,시,*,*,*,*
ㄴ다      EF,*,F,ㄴ다,*,*,*,*
EOS
```

### Node 단위 처리

```python
from mecab_ko import Tagger

tagger = Tagger()
nodes = tagger.parse_nodes("형태소 분석을 시작합니다")

for node in nodes:
    print(f"표면형: {node.surface}")
    print(f"품사: {node.pos}")
    if node.base_form:
        print(f"기본형: {node.base_form}")
    print("---")
```

### 사전 형식 출력

```python
from mecab_ko import Tagger

tagger = Tagger()
result_dict = tagger.parse_to_dict("안녕하세요")

print(result_dict["text"])  # 원본 텍스트
for node in result_dict["nodes"]:
    print(node["surface"], node["pos"])
```

### 사용자 사전 사용

```python
from mecab_ko import Tagger

tagger = Tagger(user_dict="user.csv")
result = tagger.parse("딥러닝과 머신러닝")
print(result)
```

user.csv 파일:
```csv
딥러닝,NNG,-1000,딥러닝
머신러닝,NNG,-1000,머신러닝
```

### Wakati 출력 (형태소만)

```python
from mecab_ko import Tagger

tagger = Tagger(output_format="wakati")
result = tagger.parse("형태소만 추출합니다")
print(result)  # "형태소 만 추출 하 ㅂ니다"
```

### JSON 출력

```python
from mecab_ko import Tagger
import json

tagger = Tagger(output_format="json")
result = tagger.parse("JSON 형식으로 출력")
data = json.loads(result)
print(json.dumps(data, ensure_ascii=False, indent=2))
```

### N-best 분석

```python
from mecab_ko import Tagger

tagger = Tagger()
candidates = tagger.parse_nbest("아버지가방에들어가신다", n=3)

for i, candidate in enumerate(candidates, 1):
    print(f"=== 후보 {i} ===")
    print(candidate)
```

### 대용량 파일 처리

```python
from mecab_ko import Tagger

tagger = Tagger()

with open("large_file.txt", "r", encoding="utf-8") as f:
    for line in f:
        line = line.strip()
        if line:
            result = tagger.parse(line)
            # 결과 처리
            print(result)
```

### 멀티프로세싱

```python
from mecab_ko import Tagger
from multiprocessing import Pool
import os

def analyze_text(text):
    # 각 프로세스마다 별도의 Tagger 생성
    tagger = Tagger()
    return tagger.parse(text)

if __name__ == "__main__":
    texts = [
        "첫 번째 문장",
        "두 번째 문장",
        "세 번째 문장",
    ]

    with Pool(processes=os.cpu_count()) as pool:
        results = pool.map(analyze_text, texts)

    for text, result in zip(texts, results):
        print(f"Input: {text}")
        print(f"Result: {result}\n")
```

### 데이터프레임과 통합

```python
import pandas as pd
from mecab_ko import Tagger

tagger = Tagger()

df = pd.DataFrame({
    "text": ["첫 번째 문장", "두 번째 문장", "세 번째 문장"]
})

# 형태소 분석 결과 추가
df["morphs"] = df["text"].apply(
    lambda x: [node.surface for node in tagger.parse_nodes(x)]
)

# 품사 태그 추가
df["pos_tags"] = df["text"].apply(
    lambda x: [node.pos for node in tagger.parse_nodes(x)]
)

print(df)
```

### Flask 웹 서버

```python
from flask import Flask, request, jsonify
from mecab_ko import Tagger

app = Flask(__name__)
tagger = Tagger()  # 전역으로 한 번만 생성

@app.route("/analyze", methods=["POST"])
def analyze():
    data = request.get_json()
    text = data.get("text", "")

    if not text:
        return jsonify({"error": "No text provided"}), 400

    result = tagger.parse_to_dict(text)
    return jsonify(result)

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5000)
```

사용 예:
```bash
curl -X POST http://localhost:5000/analyze \
  -H "Content-Type: application/json" \
  -d '{"text": "안녕하세요"}'
```

### 동사/형용사 추출

```python
from mecab_ko import Tagger

tagger = Tagger()
text = "저는 오늘 학교에 가서 공부를 했습니다."
nodes = tagger.parse_nodes(text)

# 동사(VV, VA) 추출
verbs = [node.surface for node in nodes if node.pos in ["VV", "VA"]]
print("동사/형용사:", verbs)
# 출력: ['가', '하']

# 명사(NNG, NNP) 추출
nouns = [node.surface for node in nodes if node.pos in ["NNG", "NNP"]]
print("명사:", nouns)
# 출력: ['오늘', '학교', '공부']
```

## 타입 힌트

Python 3.9+ 에서 타입 힌트를 사용할 수 있습니다:

```python
from mecab_ko import Tagger, Node
from typing import List, Dict, Optional

def analyze_sentences(sentences: List[str]) -> List[List[Node]]:
    tagger: Tagger = Tagger()
    results: List[List[Node]] = []

    for sentence in sentences:
        nodes: List[Node] = tagger.parse_nodes(sentence)
        results.append(nodes)

    return results
```

## 성능 최적화

### Tagger 재사용

```python
# Bad - 매번 생성
for text in texts:
    tagger = Tagger()  # 비효율적
    result = tagger.parse(text)

# Good - 재사용
tagger = Tagger()  # 한 번만 생성
for text in texts:
    result = tagger.parse(text)
```

### 배치 처리

대용량 데이터는 배치로 처리:

```python
def batch_analyze(texts, batch_size=1000):
    tagger = Tagger()
    results = []

    for i in range(0, len(texts), batch_size):
        batch = texts[i:i+batch_size]
        batch_results = [tagger.parse(text) for text in batch]
        results.extend(batch_results)

    return results
```

## 예외 처리

```python
from mecab_ko import Tagger

tagger = Tagger()

try:
    result = tagger.parse("분석할 텍스트")
    print(result)
except RuntimeError as e:
    print(f"분석 실패: {e}")
```

## 관련 문서

- [Rust API](./rust.md)
- [Node.js API](./nodejs.md)
- [CLI 사용법](../cli-usage.md)
- [사용자 사전](../user-dictionary.md)
