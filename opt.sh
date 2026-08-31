iwasi="./target/wasm32-wasip1/release-wasi/pmtxt2merged.wasm"

wasm-opt \
	-Oz \
	-o opt.wasm \
	--enable-bulk-memory \
	--enable-nontrapping-float-to-int \
	--enable-simd \
	"${iwasi}"
