base_path=./home
[[ -d "$base_path" ]] && rm -r "$base_path"

(
	cd ./ui/
	npm run build
)

mkdir "$base_path"
LOCALS=(
	"static"
	"apache"
)
for local in "${LOCALS[@]}"; do
	cp -v -R "./$local/" "$base_path/$local/"
done

(
	cd ./egoroff/
	cargo clean
	cargo b --workspace
	./target/debug/egoroff server
)
