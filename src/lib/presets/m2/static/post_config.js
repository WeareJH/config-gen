;
var xhr = new XMLHttpRequest();

xhr.open('POST', '/__bs/post');
xhr.setRequestHeader('Content-Type', 'application/json');
xhr.onload = function() {
    if (xhr.status === 200) {
        console.log('config-gen: merged RequireJS config sent');
    }
    else if (xhr.status !== 200) {
        console.log('config-gen: request failed, returned status of ' + xhr.status);
    }
};
xhr.send(JSON.stringify(requirejs.s.contexts._.config));
