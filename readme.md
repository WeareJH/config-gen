## config-gen

> Generate RequireJS Optimizer configuration for Magento 2 website based on real-world usage.

## Step 1 - Download the binary

To enable easy usage, `config-gen` is packaged as a single binary (currently only osx) - just check
the [releases](https://github.com/shakyShane/config-gen/releases) page and download the latest version.

## Step 2 - create a configuration file, call it `config-gen.yml`

```
presets:
  - name: m2
    options:
      bundle_config: file:test/fixtures/bundle-config.yml
```

## Step 3 - create the `bundle-config.yml` file as noted above.

This is what determines the parent-child relationship. This file is 
read from disk every time the `/build.json` or `/loaders.js` endpoint is
requested - which means you can navigate around the site and continue
to tweak these relationships to get the optimal result.

```yaml
bundles:
  - name: "bundles/main"
    urls:
      - "/"
      - "/index.php/women/tops-women/jackets-women.html"
    children:
      - name: "bundles/product"
        urls:
          - "/index.php/juno-jacket.html"
        children: []
```

## Step 4 - Now run the program against a Magento 2 instance

This will provide you with a new url, something like `http://127.0.0.1`. You should
exercise any areas of the frontend that you want to be covered by the optimizer.

```
./config-gen http://example.com --config config-gen.yml
```

## Step 5 - access the APIs to retrieve the generated code.

At any point, you can access the following endpoints to retrieve the generated json/js files.

|Path|Purpose|
|---|---|
|`/__bs/build.json`|Generates the configuration needed for the Optimzer|
|`/__bs/loaders.js`|Generates the JavaScript needed to load additional bundles|
|`/__bs/seed.json`|Generates a dump of the current state so that you can pick up where you left off|

---

## Using `build.json`

You'll first need to run `static-content:deploy` to ensure all assets are accessible to the optimizer - 
once you've done that, you'll need to `mv` the entire locale folder and then run the r_js tool using the 
build.json

```
mv pub/static/frontend/Acme/default/en_{GB,GB_src}
r_js -o build.json \
    baseUrl=/app/pub/static/frontend/Acme/default/en_GB_src \
    dir=/app/pub/static/frontend/Acme/default/en_GB \
    optimize=none
```

**Note 1**: the above is a short version, in the `mv` you'll probably want to check the directory
exists etc - but I'm not providing a full script here since requirements will differ.

**Note 2**: change `optimize=none` to `optimize=uglify` when you've confirmed the bundling is working.