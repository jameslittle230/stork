---
title: "Why do we encounter CORS errors?"
date: 2019-08-18
layout: post
tags: post
blurb: "An explainer describing what a Cross-Origin Resource Sharing error is, why they exist, and how to fix one once you come across it."
---

"CORS" errors are a certain type of web development errors that can be pretty confusing. _Cross-Origin Resource Sharing_ related errors pop up when the web browser is trying to protect you from potential security vulnerabilities. When you don't expect them, though, they can be pretty irritating—the error messages don't make it entirely clear what you need to do to resolve the problem, and CORS itself is a complex enough topic that you won't find any quick resolution steps on the internet.

<figure>
{% image "inspector-error.png" %}
<figcaption>An image of a CORS error in the Firefox developer tools</figcaption>
</figure>

Why do CORS errors occur in the first place? You nearly always see them when the following three cases are true: when the browser is

1. asked to load a remote resource (like a script, font, a page via AJAX, a WebGL texture, but not, notably, an image via the `<img>` tag) and that resource is
2. from an _external domain_, i.e. a domain that isn't the same as the one in your address bar, but
3. the server doesn't specify (by sending the correct HTTP headers) that the original domain is allowed to use that resource.

When all three of those cases are true, the browser will stop itself from loading the file, and will throw an error like the one above in the Javascript console. This might manifest as a font that doesn't load, an AJAX call that doesn't succeed, or other "why won't this show up" kinds of problems.

<!--more-->

**Why does the browser stop remote resources from loading?** It's a permissions issue: the browser wants to make sure that the external server has given its blessing to the website trying to load its content.

Imagine I want to steal information from a victim website. I set up an evil website that loads a script from the victim site's server. Without CORS protections in place, my evil site can download and run the victim site's script and send the victim site's data back to me. In this way, servers can protect themselves from inappropriate content access.

Unfortunately, this means that if you build a site and an external service from which it loads data, you'll have to set up this external service so that it knows it is allowed to serve data when the site is asking for it.

In the case of the [Bowdoin Orient's site](https://bowdoinorient.com), the font CDN[^1] lives on a different server from the actual site itself. That means the original page (the origin, or `https://bowdoinorient.com`) is trying to load content from a different domain (`font-cdn.bowdoinorient.co`). If we don't set up the font CDN server to allow itself to serve content to pages on `bowdoinorient.com`, the browser will refuse to load that content, meaning (in this case) our fonts will break.

My goal is to create a short guide that explains what's going on when you encounter these weird error messages, and to describe what the browser expects from the files it downloads and why.

## HTTP and Headers

Let's talk about the HyperText Transfer Protocol.

What happens when you go to a website, like `https://jameslittle.me`? On a high level, your browser sends an HTTP request (which is just a bit of text) to my server, and my server, via a program like [Apache](http://httpd.apache.org/) that runs continuously and is built to answer web requests, sends back an HTTP response with the contents of my web page.

Every time a web page loads, several of these HTTP requests are sent: the first one is for the HTML document that was requested, and the rest are for any images, scripts, fonts, or stylesheets that the HTML document says is needed. The first one is directly related to the page you asked for in your browser's address bar; any others are automatically sent by the browser as specified by the first document the browser gets back. For each request that gets sent, the server to which it gets sent responds with the data the browser asked for. Those request/response pairs make up the contents of the web page, and control what your browser displays to you.

<figure>
{% image "inspector-headers.png" %}
<figcaption>The HTTP headers for both a request (on the bottom) and a response (on the top).</figcaption>
</figure>

We can dig into the request/response pairs in greater depth by looking at them in the Web Inspector.[^2] Each request and response has two parts: the _headers_ and the _payload_.

**Headers:** The headers define configuration and other metadata for the message. They are plain-text key-value pairs that go at the beginning of both HTTP requests and responses. HTTP request headers are messages that the _browser_ wants to tell the _server_, while HTTP response headers are messages that the _server_ wants to tell the _browser_.

**Payload:** The payload for an HTTP _response_ will almost always be the contents of the requested file. HTTP requests can sometimes have a payload, though most of the time this payload is empty — usually, an HTTP request only consists of headers describing the file the browser is asking for.

<div class="note">

For more information about what HTTP requests and responses look like (and what they can do), [Julia Evans](https://jvns.ca) has a zine coming out that does a fantastic job explaining it. When she publishes it, I'll update this post with the link here.

Update: Julia has been tweeting about HTTP like crazy! Here are some tweets:

[Using HTTP APIs](https://twitter.com/b0rk/status/1160933788949655552), [HTTP headers](https://twitter.com/b0rk/status/1164181027469832196), [HTTP Response headers](https://twitter.com/b0rk/status/1161262574031265793), [Security headers](https://twitter.com/b0rk/status/1160185182323970050), [custom headers](https://twitter.com/b0rk/status/1161283690925834241), Request methods [part one](https://twitter.com/b0rk/status/1161679906415218690) and [part two](https://twitter.com/b0rk/status/1161680137865367553), [HTTP request methods](https://twitter.com/b0rk/status/1161679906415218690), [the Same Origin policy](https://twitter.com/b0rk/status/1155493682885341184) and [why it matters](https://twitter.com/b0rk/status/1163460967067541504).

Update 2: [Here is the zine!](https://wizardzines.com/zines/http/)

</div>

## How HTTP headers relate to CORS errors

As I described earlier, CORS errors occur when the server hasn't specified that the browser is allowed to load the resource. That permission is communicated using an HTTP header on the response: the server will add a header that says "If any page on `bowdoinorient.com` downloads a file from me, it is allowed to use it."

Remember: HTTP headers are key-value pairs in the beginning of an HTTP message. The one that describes this permission granting has a key of **`Access-Control-Allow-Origin`**, and has a value of the _origin_ allowed to use that file (in this example, **`bowdoinorient.com`**). If that key-value pair is present on an HTTP response from an external server (like a font CDN), any time a page on `bowdoinorient.com` wants to load that font, the browser will allow that to happen.

<div class="note">

In the case of `GET` requests, the file is always downloaded, but if the browser finds itself in a situation that would break the CORS policy, it will refuse to load the file's contents: the script won't run, the stylesheet won't get used, etc.

`POST` requests are different: the browser typically sends a canary request (called an `OPTIONS` request) to check what it's allowed to do, and if it finds it is allowed to make the POST request, it does so.

</div>

Therefore, if you're getting console warnings about CORS headers not being properly included, it means you have to change the configuration on your server: your server needs to be including HTTP headers in the response so that the browser knows it's allowed to use the file it downloaded.

## What kinds of headers should I include?

It sort of depends on what your browser is asking for — while the console error messages might not immediately be clear, you can usually tell which header is missing from the error message. In the example above, the server needs to attach the **`Access-Control-Allow-Origin`** header with a value that says that `bowdoinorient.com` pages are allowed to use the font file.

I mentioned above that the header's value can be set as **`bowdoinorient.com`**, and that will allow pages on `bowdoinorient.com` to load the resource. But we could also configure the value to be **`*`**, which would specify that any site is allowed to use that resource.[^4]

CORS errors can manifest in different ways, since there are different permissions that a server can specify. There are more headers that give more granular permissions to browsers. These headers might include:

**`Access-Control-Allow-Methods`**: describes which HTTP methods (GET, POST, PUT, DELETE, etc.) are allowed to be used on a given URI. When you use Javascript to make an AJAX request, sometimes it will send a _preflight request_: an additional request beforehand to see what sorts of requests the browser is allowed to make before it actually makes the request. The server will respond with this header to let the browser (and, ultimately, you) know what kinds of HTTP methods you can use next.

**`Access-Control-Allow-Headers`**: describes which request headers are allowed to be sent while asking for a given resource. For example, the browser (again, through Javascript) might specify in a request header that it wants JSON-only responses (`content-type: JSON` would be the header tacked onto the HTTP request). If you send a server a header it doesn't expect, it might reject the request altogether.

## Conclusion

When struggling with CORS errors, the concepts I always have to remind myself are:

1. The file is usually being properly downloaded, but the browser is blocking the file from being used
2. The server needs to be changed to give the browser permission to use that file
3. That change needs to be a new header that gets included with the HTTP response

Those three concepts are the biggies. And ultimately I always feel like the fix was something simple that just requires a large research journey. Security is just like that, I guess.

<div class="note">

[Update: I wrote about a CORS error that I encountered and fixed.](/blog/2019/cors-debug/)

</div>

<!-- Footnotes -->

[^1]: Content Distribution Network: another server whose job it is to cache and serve static files very quickly.
[^2]: I use Firefox, so that's where these inspector screenshots come from. But every reasonable web browser has an inspector these days, and all of them let you look at the contents and headers of an HTTP request and response.
[^3]: If you're reading this to sort out a CORS error you're having, your web browser does not have the right permission because the server has not given it to the browser.
[^4]: General purpose CDNs, like Google Fonts, will have this `*` as the value for their `Access-Control-Allow-Origin` header.
