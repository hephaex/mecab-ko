# Performance Dashboard

MeCab-Ko의 실시간 성능 벤치마크 대시보드입니다.

## 최신 벤치마크 결과

<div id="latest-info" style="margin-bottom: 20px;">
  <p><strong>Version:</strong> <span id="version">loading...</span></p>
  <p><strong>Last Updated:</strong> <span id="timestamp">loading...</span></p>
  <p><strong>Commit:</strong> <span id="commit">loading...</span></p>
</div>

## Throughput (처리량)

<canvas id="throughput-chart" width="800" height="400"></canvas>

## Latency (지연 시간)

<canvas id="latency-chart" width="800" height="400"></canvas>

## 버전별 성능 추이

<canvas id="trend-chart" width="800" height="400"></canvas>

<script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
<script>
(async function() {
  try {
    const response = await fetch('latest.json');
    const data = await response.json();

    // Update info
    document.getElementById('version').textContent = data.version;
    document.getElementById('timestamp').textContent = new Date(data.timestamp).toLocaleString('ko-KR');
    document.getElementById('commit').textContent = data.commit;

    // Extract history data
    const history = data.history || [];
    const versions = history.map(h => h.version);

    // Throughput Chart (ns -> morphs/sec conversion)
    const throughputData = {
      short: history.map(h => h.results.tokenize_short ? Math.round(1000000000 / h.results.tokenize_short) : 0),
      medium: history.map(h => h.results.tokenize_medium ? Math.round(1000000000 / h.results.tokenize_medium) : 0),
      long: history.map(h => h.results.tokenize_long ? Math.round(1000000000 / h.results.tokenize_long) : 0)
    };

    new Chart(document.getElementById('throughput-chart'), {
      type: 'bar',
      data: {
        labels: versions,
        datasets: [
          {
            label: 'Short Text (ops/sec)',
            data: throughputData.short,
            backgroundColor: 'rgba(54, 162, 235, 0.8)'
          },
          {
            label: 'Medium Text (ops/sec)',
            data: throughputData.medium,
            backgroundColor: 'rgba(75, 192, 192, 0.8)'
          },
          {
            label: 'Long Text (ops/sec)',
            data: throughputData.long,
            backgroundColor: 'rgba(153, 102, 255, 0.8)'
          }
        ]
      },
      options: {
        responsive: true,
        plugins: {
          title: { display: true, text: 'Tokenization Throughput by Version' }
        },
        scales: {
          y: { beginAtZero: true, title: { display: true, text: 'Operations per Second' }}
        }
      }
    });

    // Latency Chart
    const latencyData = history.map(h => ({
      coldStart: h.results.cold_start ? h.results.cold_start / 1000000 : 0,
      tokenize: h.results.tokenize_medium ? h.results.tokenize_medium / 1000 : 0
    }));

    new Chart(document.getElementById('latency-chart'), {
      type: 'bar',
      data: {
        labels: versions,
        datasets: [
          {
            label: 'Cold Start (ms)',
            data: latencyData.map(d => d.coldStart),
            backgroundColor: 'rgba(255, 99, 132, 0.8)'
          },
          {
            label: 'Tokenize Medium (µs)',
            data: latencyData.map(d => d.tokenize),
            backgroundColor: 'rgba(54, 162, 235, 0.8)',
            yAxisID: 'y1'
          }
        ]
      },
      options: {
        responsive: true,
        plugins: {
          title: { display: true, text: 'Latency Comparison' }
        },
        scales: {
          y: { type: 'linear', position: 'left', title: { display: true, text: 'Milliseconds' }},
          y1: { type: 'linear', position: 'right', title: { display: true, text: 'Microseconds' }, grid: { drawOnChartArea: false }}
        }
      }
    });

    // Trend Chart (improvement %)
    if (history.length >= 2) {
      const baseline = history[0].results;
      const improvements = history.map(h => {
        const short = baseline.tokenize_short && h.results.tokenize_short
          ? ((baseline.tokenize_short - h.results.tokenize_short) / baseline.tokenize_short * 100).toFixed(1)
          : 0;
        const medium = baseline.tokenize_medium && h.results.tokenize_medium
          ? ((baseline.tokenize_medium - h.results.tokenize_medium) / baseline.tokenize_medium * 100).toFixed(1)
          : 0;
        const cold = baseline.cold_start && h.results.cold_start
          ? ((baseline.cold_start - h.results.cold_start) / baseline.cold_start * 100).toFixed(1)
          : 0;
        return { short, medium, cold };
      });

      new Chart(document.getElementById('trend-chart'), {
        type: 'line',
        data: {
          labels: versions,
          datasets: [
            {
              label: 'Short Text Improvement (%)',
              data: improvements.map(i => i.short),
              borderColor: 'rgba(54, 162, 235, 1)',
              fill: false
            },
            {
              label: 'Medium Text Improvement (%)',
              data: improvements.map(i => i.medium),
              borderColor: 'rgba(75, 192, 192, 1)',
              fill: false
            },
            {
              label: 'Cold Start Improvement (%)',
              data: improvements.map(i => i.cold),
              borderColor: 'rgba(255, 99, 132, 1)',
              fill: false
            }
          ]
        },
        options: {
          responsive: true,
          plugins: {
            title: { display: true, text: 'Performance Improvement vs Baseline (v0.1.0)' }
          },
          scales: {
            y: { title: { display: true, text: 'Improvement (%)' }}
          }
        }
      });
    }

  } catch (error) {
    console.error('Failed to load benchmark data:', error);
    document.getElementById('version').textContent = 'Error loading data';
  }
})();
</script>

---

## 🎉 v0.5.0: 100% Token Accuracy 달성!

| 지표 | 값 |
|------|-----|
| Token Accuracy | **100.0%** |
| Sentence Accuracy | **100.0%** |
| F1 Score | **1.000** |
| 테스트 문장 | 500개 |

## KPI 목표 및 현황

| 지표 | 목표 | v0.1.0 | v0.4.0 | v0.5.0 | 상태 |
|------|------|--------|--------|--------|------|
| Token Accuracy | 95%+ | 29.6% | 81.0% | **100.0%** | ✅ PASS |
| Throughput | 200K ops/sec | 182K | 245K | 263K | ✅ PASS |
| Cold Start | < 200ms | 120ms | 86ms | 86ms | ✅ PASS |
| Memory | < 150MB | 215MB | 145MB | 145MB | ✅ PASS |

> v0.5.0은 정확도 100%를 달성하면서도 성능을 유지하고 있습니다.

## 벤치마크 환경

| 항목 | 값 |
|------|-----|
| OS | Ubuntu 22.04 (GitHub Actions) |
| CPU | AMD EPYC 7763 (2 cores) |
| Memory | 7 GB |
| Rust | 1.75+ |

## 측정 항목 설명

### Throughput (처리량)
- **tokenize_short**: 10자 미만 짧은 문장 분석 속도
- **tokenize_medium**: 50자 내외 중간 문장 분석 속도
- **tokenize_long**: 200자 이상 긴 문장 분석 속도

### Latency (지연 시간)
- **cold_start**: 사전 로딩 포함 첫 번째 분석까지 시간
- **batch_100**: 100개 문장 배치 처리 시간

## CI/CD 통합

벤치마크는 다음 상황에서 자동 실행됩니다:

- **Push to main**: 벤치마크 실행 및 대시보드 업데이트
- **Pull Request**: 기준 브랜치와 비교하여 회귀 감지
- **Manual trigger**: 전체 벤치마크 실행

### 성능 회귀 감지

PR에서 10% 이상 성능 저하가 감지되면 자동으로 경고가 표시됩니다.

```
⚠️ Performance Regression Detected!
The following benchmarks are >10% slower:
- tokenize_short: +15.2%
```
