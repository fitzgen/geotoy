# Geotoy

To run natively:

	cargo run --features=glium

Running in the browser requires a few initial steps:

	cd wasm
	npm install
	npm run update

Then, to compile and serve on [localhost:8080](http://localhost:8080):

	npm run build
	npm run serve
