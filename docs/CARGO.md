# Cargo 설정 가이드

## Cargo.toml 구조

```toml
[package]
name = "notion-cli-rs"
version = "0.1.0"
edition = "2021"

[dependencies]
# CLI 파싱
clap = { version = "4", features = ["derive"] }

# HTTP 클라이언트
reqwest = { version = "0.12", default-features = false, features = ["json", "blocking", "rustls-tls"] }

# JSON 처리
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# 환경 변수
dotenvy = "0.15"

# 에러 처리
anyhow = "1"

# 터미널 색상
colored = "2"

[[bin]]
name = "notion"
path = "src/main.rs"
```

## 의존성 설명

### clap
- **용도**: 명령줄 인자 파싱
- **features = ["derive"]**: `#[derive(Parser)]` 매크로 사용

### reqwest
- **용도**: HTTP 클라이언트
- **default-features = false**: 불필요한 기능 제외 (빌드 시간 단축)
- **features**:
  - `json`: JSON 직렬화/역직렬화
  - `blocking`: 동기 API (async 불필요)
  - `rustls-tls`: OpenSSL 대신 rustls 사용 (의존성 감소)

### serde / serde_json
- **용도**: JSON 처리
- **features = ["derive"]**: `#[derive(Serialize, Deserialize)]` 사용 가능

### dotenvy
- **용도**: `.env` 파일 로드
- **참고**: `dotenv` 크레이트의 유지보수되는 포크

### anyhow
- **용도**: 간편한 에러 처리
- **특징**: `?` 연산자로 에러 체이닝, `.context()` 추가

### colored
- **용도**: 터미널 색상 출력
- **예시**: `"text".green()`, `"error".red()`

## 빌드 최적화

### 개발 빌드
```bash
cargo build
```
- 빠른 컴파일
- 디버그 심볼 포함
- 최적화 없음

### 릴리스 빌드
```bash
cargo build --release
```
- 최적화 활성화 (`opt-level = 3`)
- 디버그 심볼 제거
- 더 작고 빠른 바이너리

### 추가 최적화 (선택)

`Cargo.toml`에 추가 가능:

```toml
[profile.release]
opt-level = "z"     # 크기 최적화 (속도 대신)
lto = true          # Link Time Optimization
codegen-units = 1   # 단일 코드젠 유닛 (더 나은 최적화)
panic = "abort"     # 패닉 시 언와인딩 없이 종료
strip = true        # 심볼 스트립
```

## 유용한 Cargo 명령어

```bash
# 의존성 트리 확인
cargo tree

# 오래된 의존성 확인
cargo outdated  # cargo-outdated 설치 필요

# 보안 취약점 검사
cargo audit  # cargo-audit 설치 필요

# 빌드 캐시 정리
cargo clean

# 문서 생성
cargo doc --open

# 테스트 실행
cargo test

# 린트 (clippy)
cargo clippy

# 포맷팅
cargo fmt
```

## 캐시 관리

### target 디렉토리
- 빌드 아티팩트 저장 위치
- `.gitignore`에 포함됨
- `cargo clean`으로 정리

### ~/.cargo
- 글로벌 캐시 (다운로드된 크레이트)
- 보통 정리 불필요
- 필요시: `rm -rf ~/.cargo/registry/cache`

## 트러블슈팅

### OpenSSL 에러
`rustls-tls` 피처를 사용하므로 OpenSSL 불필요.
만약 네이티브 TLS 필요시:
```bash
# Ubuntu/Debian
sudo apt install pkg-config libssl-dev
```

### 빌드 시간이 오래 걸릴 때
1. `cargo build --release` 대신 `cargo build` 사용 (개발 중)
2. `sccache` 설치하여 컴파일 캐시 활용
3. `mold` 링커 사용 (Linux)

```bash
# mold 링커 사용
RUSTFLAGS="-C link-arg=-fuse-ld=mold" cargo build
```
