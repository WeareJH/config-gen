## config-gen [![Build Status](https://travis-ci.org/WeareJH/config-gen.svg?branch=master)](https://travis-ci.org/WeareJH/config-gen)

> Generate RequireJS Optimizer configuration for Magento 2 website based on real-world usage.


**Note**: The following is just a **static** representation of a Magento 2 homepage. It's there just to illustrate
the performance benefits that are possible when using the RequireJS Optimizer

https://laughing-darwin-eb1b63.netlify.com/

You can also look at the following 10 commits to see exactly how I reached a score 83 on Lighthouse https://github.com/shakyShane/m2-static/commits/master

---

## Step 1 - Download the binary

To enable easy usage, `config-gen` is packaged as a single binary (currently only osx) - just check
the [releases](https://github.com/shakyShane/config-gen/releases) page and download the latest version.

To make it executable, run

```sh
chmod +x config-gen
```

**Note**: Some easier installation methods will be released in the coming weeks. :)

## Step 2 - create a configuration file (either `json` or `yaml`)

You can name it anything, but for example, this would be config-gen.yml

```yml
presets:
  - name: m2
    options:
      bundle_config: file:test/fixtures/bundle-config.yml
```

<details>
  <summary>See <code>.json</code> version</summary><p>

    {
      "presets": [
        {
          "name": "m2",
          "options": {
            "bundle_config": "file:test/fixtures/bundle-config.json"
          }
        }
      ]
    }

</p>
</details>

## Step 3 - create the `bundle-config.yml` or `json` file as noted above.

This is what determines the parent-child relationship. This file is 
read from disk every time the `/build.json` or `/loaders.js` endpoint is
requested - which means you can navigate around the site and continue
to tweak these relationships to get the optimal result.

```yml
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

<details>
  <summary>See <code>.json</code> version</summary><p>

    {
      "bundles": [
        {
          "name": "bundles/main",
          "urls": [
            "/",
            "/index.php/women/tops-women/jackets-women.html"
          ],
          "children": [
            {
              "name": "bundles/product",
              "urls": [
                "/index.php/juno-jacket.html"
              ],
              "children": []
            }
          ]
        }
      ]
    }

</p>
</details>

## Step 4 - Now run the program against a Magento 2 instance

This will provide you with a new url, something like `http://127.0.0.1:65432`. You should
exercise any areas of the frontend that you want to be covered by the optimizer.

*Tip:* You'll want to disable the network cache, either do this by opening dev-tools and checking
the 'disable network cache' checkbox, or you can open your browser with a specific flag, eg: in chrome
you can run `/Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome --disk-cache-size=1`

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

---

## Loading additional bundles

When you access `/__bs/loaders.json`, it will provide Javascript snippets that will allow the additional bundles
to be loaded when needed - exactly how you implement this part is up to you, however we will soon be providing a reference
implementation that shows how to add/remove the additional bundles based on some admin flags.

```html
<script src="http://example.com/../requirejs/require.js"></script>
<script src="http://example.com/../mage/requirejs/mixins.js"></script>
<script src="http://example.com/../requirejs-config.js"></script>
<script src="http://example.com/../default-loader.js"></script> <!-- <<<< this is the new file -->
```

Where the contents of `default-loader.js` is obtained from the config-gen api `/__bs/loaders.js`.

The key point here though, is that during development you wont want that bundle file in place, so this file
should be conditionally added via xml config when in production.
