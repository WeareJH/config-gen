({
  "dir": "static/frontend/Magento/luma/en_US",
  "baseUrl": "static/frontend/Magento/luma/en_US_src",
  "optimize": "none",
  "generateSourceMaps": true,
  "keepBuildDir": true,
  "paths": {
    "jquery/ui": "jquery/jquery-ui",
    "jquery/validate": "jquery/jquery.validate",
    "jquery/hover-intent": "jquery/jquery.hoverIntent",
    "jquery/file-uploader": "jquery/fileUploader/jquery.fileupload-fp",
    "jquery/jquery.hashchange": "jquery/jquery.ba-hashchange.min",
    "prototype": "legacy-build.min",
    "jquery/jquery-storageapi": "jquery/jquery.storageapi.min",
    "text": "mage/requirejs/text",
    "domReady": "requirejs/domReady",
    "tinymce": "tiny_mce/tiny_mce_src",
    "ui/template": "Magento_Ui/templates",
    "paypalInContextExpressCheckout": "empty:",
    "trackingCode": "Dotdigitalgroup_Email/js/trackingCode",
    "temandoCheckoutFieldsDefinition": "Temando_Shipping/js/model/fields-definition",
    "temandoCustomerAddressRateProcessor": "Temando_Shipping/js/model/shipping-rate-processor/customer-address",
    "temandoNewAddressRateProcessor": "Temando_Shipping/js/model/shipping-rate-processor/new-address",
    "temandoShippingRatesValidator": "Temando_Shipping/js/model/shipping-rates-validator/temando",
    "temandoShippingRatesValidationRules": "Temando_Shipping/js/model/shipping-rates-validation-rules/temando",
    "mixins": "mage/requirejs/mixins"
  },
  "bundles": {},
  "pkgs": {},
  "shim": {
    "jquery/jquery-migrate": {
      "deps": [
        "jquery"
      ]
    },
    "jquery/jquery.hashchange": {
      "deps": [
        "jquery",
        "jquery/jquery-migrate"
      ]
    },
    "jquery/jstree/jquery.hotkeys": {
      "deps": [
        "jquery"
      ]
    },
    "jquery/hover-intent": {
      "deps": [
        "jquery"
      ]
    },
    "mage/adminhtml/backup": {
      "deps": [
        "prototype"
      ]
    },
    "mage/captcha": {
      "deps": [
        "prototype"
      ]
    },
    "mage/common": {
      "deps": [
        "jquery"
      ]
    },
    "mage/new-gallery": {
      "deps": [
        "jquery"
      ]
    },
    "mage/webapi": {
      "deps": [
        "jquery"
      ]
    },
    "jquery/ui": {
      "deps": [
        "jquery"
      ]
    },
    "MutationObserver": {
      "deps": [
        "es6-collections"
      ]
    },
    "tinymce": {
      "exports": "tinymce"
    },
    "moment": {
      "exports": "moment"
    },
    "matchMedia": {
      "exports": "mediaCheck"
    },
    "jquery/jquery-storageapi": {
      "deps": [
        "jquery/jquery.cookie"
      ]
    },
    "vimeoAPI": {},
    "paypalInContextExpressCheckout": {
      "exports": "paypal"
    },
    "trackingCode": {
      "exports": "_dmTrack"
    }
  },
  "config": {
    "mixins": {
      "jquery/jstree/jquery.jstree": {
        "mage/backend/jstree-mixin": true
      },
      "Magento_Checkout/js/action/place-order": {
        "Magento_CheckoutAgreements/js/model/place-order-mixin": true
      },
      "Magento_Checkout/js/action/set-payment-information": {
        "Magento_CheckoutAgreements/js/model/set-payment-information-mixin": true
      }
    },
    "text": {
      "headers": {
        "X-Requested-With": "XMLHttpRequest"
      }
    }
  },
  "map": {
    "*": {
      "rowBuilder": "Magento_Theme/js/row-builder",
      "toggleAdvanced": "mage/toggle",
      "translateInline": "mage/translate-inline",
      "sticky": "mage/sticky",
      "tabs": "mage/tabs",
      "zoom": "mage/zoom",
      "collapsible": "mage/collapsible",
      "dropdownDialog": "mage/dropdown",
      "dropdown": "mage/dropdowns",
      "accordion": "mage/accordion",
      "loader": "mage/loader",
      "tooltip": "mage/tooltip",
      "deletableItem": "mage/deletable-item",
      "itemTable": "mage/item-table",
      "fieldsetControls": "mage/fieldset-controls",
      "fieldsetResetControl": "mage/fieldset-controls",
      "redirectUrl": "mage/redirect-url",
      "loaderAjax": "mage/loader",
      "menu": "mage/menu",
      "popupWindow": "mage/popup-window",
      "validation": "mage/validation/validation",
      "welcome": "Magento_Theme/js/view/welcome",
      "ko": "knockoutjs/knockout",
      "knockout": "knockoutjs/knockout",
      "mageUtils": "mage/utils/main",
      "rjsResolver": "mage/requirejs/resolver",
      "checkoutBalance": "Magento_Customer/js/checkout-balance",
      "address": "Magento_Customer/address",
      "changeEmailPassword": "Magento_Customer/change-email-password",
      "passwordStrengthIndicator": "Magento_Customer/js/password-strength-indicator",
      "zxcvbn": "Magento_Customer/js/zxcvbn",
      "addressValidation": "Magento_Customer/js/addressValidation",
      "compareList": "Magento_Catalog/js/list",
      "relatedProducts": "Magento_Catalog/js/related-products",
      "upsellProducts": "Magento_Catalog/js/upsell-products",
      "productListToolbarForm": "Magento_Catalog/js/product/list/toolbar",
      "catalogGallery": "Magento_Catalog/js/gallery",
      "priceBox": "Magento_Catalog/js/price-box",
      "priceOptionDate": "Magento_Catalog/js/price-option-date",
      "priceOptionFile": "Magento_Catalog/js/price-option-file",
      "priceOptions": "Magento_Catalog/js/price-options",
      "priceUtils": "Magento_Catalog/js/price-utils",
      "catalogAddToCart": "Magento_Catalog/js/catalog-add-to-cart",
      "addToCart": "Magento_Msrp/js/msrp",
      "quickSearch": "Magento_Search/form-mini",
      "bundleOption": "Magento_Bundle/bundle",
      "priceBundle": "Magento_Bundle/js/price-bundle",
      "slide": "Magento_Bundle/js/slide",
      "productSummary": "Magento_Bundle/js/product-summary",
      "creditCardType": "Magento_Payment/cc-type",
      "downloadable": "Magento_Downloadable/downloadable",
      "giftMessage": "Magento_Sales/gift-message",
      "ordersReturns": "Magento_Sales/orders-returns",
      "catalogSearch": "Magento_CatalogSearch/form-mini",
      "requireCookie": "Magento_Cookie/js/require-cookie",
      "cookieNotices": "Magento_Cookie/js/notices",
      "discountCode": "Magento_Checkout/js/discount-codes",
      "shoppingCart": "Magento_Checkout/js/shopping-cart",
      "regionUpdater": "Magento_Checkout/js/region-updater",
      "sidebar": "Magento_Checkout/js/sidebar",
      "checkoutLoader": "Magento_Checkout/js/checkout-loader",
      "checkoutData": "Magento_Checkout/js/checkout-data",
      "proceedToCheckout": "Magento_Checkout/js/proceed-to-checkout",
      "taxToggle": "Magento_Weee/tax-toggle",
      "giftOptions": "Magento_GiftMessage/gift-options",
      "extraOptions": "Magento_GiftMessage/extra-options",
      "uiElement": "Magento_Ui/js/lib/core/element/element",
      "uiCollection": "Magento_Ui/js/lib/core/collection",
      "uiComponent": "Magento_Ui/js/lib/core/collection",
      "uiClass": "Magento_Ui/js/lib/core/class",
      "uiEvents": "Magento_Ui/js/lib/core/events",
      "uiRegistry": "Magento_Ui/js/lib/registry/registry",
      "consoleLogger": "Magento_Ui/js/lib/logger/console-logger",
      "uiLayout": "Magento_Ui/js/core/renderer/layout",
      "buttonAdapter": "Magento_Ui/js/form/button-adapter",
      "configurable": "Magento_ConfigurableProduct/js/configurable",
      "multiShipping": "Magento_Multishipping/js/multi-shipping",
      "orderOverview": "Magento_Multishipping/js/overview",
      "payment": "Magento_Multishipping/js/payment",
      "recentlyViewedProducts": "Magento_Reports/js/recently-viewed",
      "pageCache": "Magento_PageCache/js/page-cache",
      "loadPlayer": "Magento_ProductVideo/js/load-player",
      "fotoramaVideoEvents": "Magento_ProductVideo/js/fotorama-add-video-events",
      "orderReview": "Magento_Paypal/order-review",
      "paypalCheckout": "Magento_Paypal/js/paypal-checkout",
      "transparent": "Magento_Payment/transparent",
      "captcha": "Magento_Captcha/captcha",
      "wishlist": "Magento_Wishlist/js/wishlist",
      "addToWishlist": "Magento_Wishlist/js/add-to-wishlist",
      "wishlistSearch": "Magento_Wishlist/js/search",
      "editTrigger": "mage/edit-trigger",
      "addClass": "Magento_Translation/add-class",
      "braintree": "https://js.braintreegateway.com/js/braintree-2.32.0.min.js"
    },
    "Magento_Checkout/js/model/shipping-rate-service": {
      "Magento_Checkout/js/model/shipping-rate-processor/customer-address": "temandoCustomerAddressRateProcessor",
      "Magento_Checkout/js/model/shipping-rate-processor/new-address": "temandoNewAddressRateProcessor"
    }
  },
  "deps": [
    "mage/requirejs/mixins",
    "Dotdigitalgroup_Email/js/emailCapture",
    "FormData",
    "Magento_Captcha/js/action/refresh",
    "Magento_Captcha/js/model/captcha",
    "Magento_Captcha/js/model/captchaList",
    "Magento_Captcha/js/view/checkout/defaultCaptcha",
    "Magento_Captcha/js/view/checkout/loginCaptcha",
    "Magento_Catalog/js/product/query-builder",
    "Magento_Catalog/js/product/storage/data-storage",
    "Magento_Catalog/js/product/storage/ids-storage",
    "Magento_Catalog/js/product/storage/ids-storage-compare",
    "Magento_Catalog/js/product/storage/storage-service",
    "Magento_Catalog/js/storage-manager",
    "Magento_Catalog/js/view/compare-products",
    "Magento_Catalog/js/view/image",
    "Magento_Checkout/js/sidebar",
    "Magento_Checkout/js/view/minicart",
    "Magento_Customer/js/action/login",
    "Magento_Customer/js/customer-data",
    "Magento_Customer/js/invalidation-processor",
    "Magento_Customer/js/invalidation-rules/website-rule",
    "Magento_Customer/js/model/authentication-popup",
    "Magento_Customer/js/section-config",
    "Magento_Customer/js/view/authentication-popup",
    "Magento_Customer/js/view/customer",
    "Magento_Msrp/js/view/checkout/minicart/subtotal/totals",
    "Magento_PageCache/js/page-cache",
    "Magento_Paypal/js/in-context/express-checkout",
    "Magento_Search/form-mini",
    "Magento_Tax/js/view/checkout/minicart/subtotal/totals",
    "Magento_Theme/js/responsive",
    "Magento_Theme/js/theme",
    "Magento_Theme/js/view/messages",
    "Magento_Ui/js/block-loader",
    "Magento_Ui/js/core/app",
    "Magento_Ui/js/core/renderer/layout",
    "Magento_Ui/js/core/renderer/types",
    "Magento_Ui/js/form/adapter",
    "Magento_Ui/js/form/form",
    "Magento_Ui/js/lib/core/class",
    "Magento_Ui/js/lib/core/collection",
    "Magento_Ui/js/lib/core/element/element",
    "Magento_Ui/js/lib/core/element/links",
    "Magento_Ui/js/lib/core/events",
    "Magento_Ui/js/lib/core/storage/local",
    "Magento_Ui/js/lib/key-codes",
    "Magento_Ui/js/lib/knockout/bindings/after-render",
    "Magento_Ui/js/lib/knockout/bindings/autoselect",
    "Magento_Ui/js/lib/knockout/bindings/bind-html",
    "Magento_Ui/js/lib/knockout/bindings/bootstrap",
    "Magento_Ui/js/lib/knockout/bindings/collapsible",
    "Magento_Ui/js/lib/knockout/bindings/datepicker",
    "Magento_Ui/js/lib/knockout/bindings/fadeVisible",
    "Magento_Ui/js/lib/knockout/bindings/i18n",
    "Magento_Ui/js/lib/knockout/bindings/keyboard",
    "Magento_Ui/js/lib/knockout/bindings/mage-init",
    "Magento_Ui/js/lib/knockout/bindings/optgroup",
    "Magento_Ui/js/lib/knockout/bindings/outer_click",
    "Magento_Ui/js/lib/knockout/bindings/range",
    "Magento_Ui/js/lib/knockout/bindings/resizable",
    "Magento_Ui/js/lib/knockout/bindings/scope",
    "Magento_Ui/js/lib/knockout/bindings/simple-checked",
    "Magento_Ui/js/lib/knockout/bindings/staticChecked",
    "Magento_Ui/js/lib/knockout/bindings/tooltip",
    "Magento_Ui/js/lib/knockout/bootstrap",
    "Magento_Ui/js/lib/knockout/extender/bound-nodes",
    "Magento_Ui/js/lib/knockout/extender/observable_array",
    "Magento_Ui/js/lib/knockout/template/engine",
    "Magento_Ui/js/lib/knockout/template/loader",
    "Magento_Ui/js/lib/knockout/template/observable_source",
    "Magento_Ui/js/lib/knockout/template/renderer",
    "Magento_Ui/js/lib/logger/console-logger",
    "Magento_Ui/js/lib/logger/console-output-handler",
    "Magento_Ui/js/lib/logger/entry",
    "Magento_Ui/js/lib/logger/entry-factory",
    "Magento_Ui/js/lib/logger/formatter",
    "Magento_Ui/js/lib/logger/levels-pool",
    "Magento_Ui/js/lib/logger/logger",
    "Magento_Ui/js/lib/logger/logger-utils",
    "Magento_Ui/js/lib/logger/message-pool",
    "Magento_Ui/js/lib/registry/registry",
    "Magento_Ui/js/lib/spinner",
    "Magento_Ui/js/lib/view/utils/async",
    "Magento_Ui/js/lib/view/utils/bindings",
    "Magento_Ui/js/lib/view/utils/dom-observer",
    "Magento_Ui/js/modal/alert",
    "Magento_Ui/js/modal/confirm",
    "Magento_Ui/js/modal/modal",
    "Magento_Ui/js/model/messageList",
    "Magento_Ui/js/model/messages",
    "Magento_Ui/js/view/messages",
    "MutationObserver",
    "domReady",
    "es6-collections",
    "jquery",
    "jquery/jquery-migrate",
    "jquery/jquery-storageapi",
    "jquery/jquery-ui-timepicker-addon",
    "jquery/jquery.cookie",
    "jquery/jquery.metadata",
    "jquery/jquery.mobile.custom",
    "jquery/ui",
    "jquery/validate",
    "knockoutjs/knockout",
    "knockoutjs/knockout-es5",
    "knockoutjs/knockout-fast-foreach",
    "knockoutjs/knockout-repeat",
    "mage/apply/main",
    "mage/apply/scripts",
    "mage/bootstrap",
    "mage/calendar",
    "mage/collapsible",
    "mage/common",
    "mage/cookies",
    "mage/dataPost",
    "mage/decorate",
    "mage/dropdown",
    "mage/dropdowns",
    "mage/ie-class-fixer",
    "mage/loader",
    "mage/mage",
    "mage/menu",
    "mage/redirect-url",
    "mage/requirejs/resolver",
    "mage/smart-keyboard-handler",
    "mage/storage",
    "mage/tabs",
    "mage/template",
    "mage/translate",
    "mage/translate-inline",
    "mage/url",
    "mage/utils/arrays",
    "mage/utils/compare",
    "mage/utils/main",
    "mage/utils/misc",
    "mage/utils/objects",
    "mage/utils/strings",
    "mage/utils/template",
    "mage/utils/wrapper",
    "mage/validation",
    "mage/validation/validation",
    "matchMedia",
    "moment",
    "text",
    "trackingCode",
    "underscore"
  ],
  "modules": [
    {
      "name": "requirejs/require"
    }
  ]
})