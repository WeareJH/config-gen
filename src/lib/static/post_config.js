var xhr = new XMLHttpRequest();

xhr.open('POST', '/__bs/post');
xhr.setRequestHeader('Content-Type', 'application/json');
xhr.onload = function() {
    if (xhr.status === 200) {
        console.log('sent');
    }
    else if (xhr.status !== 200) {
        alert('Request failed.  Returned status of ' + xhr.status);
    }
};
xhr.send(JSON.stringify(requirejs.s.contexts._.config));
