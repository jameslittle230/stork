---
title: "Building Internal Analytics for Stork"
date: 2021-06-02
layout: post
tags: post
blurb: "A Docker love letter? An admission that I'm bad at dev-ops? Maybe I just did something I want to brag about. This is that brag."
---

I wrote a little Node application that downloads all the Cloudfront logs that [Stork](https://stork-search.net) generates, and sticks the data in a SQLite database. I stuck a Dockerfile in front of that application. Then, I added another directory that has a Dockerfile pointing to [Datasette](https://datasette.io). Now my project is a monorepo that contains multiple services.

A monorepo? Services? Complexity has skyrocketed.

I started up an EC2 box. I installed Docker on it, and I set up `docker-compose` and a crontab so that this box does two things: it serves my Datasette instance on port 80, and it runs the Node application on a cron job. Now, every 6 hours, my Datasette instance updates with the latest usage stats for Stork, and I can use the Datasette web instance from anywhere.

I can write SQL queries and get stats about the HTTP requests coming into the Stork CDN. It's my own little data warehouse! Mission accomplished.

---

I think [Observable](https://observablehq.com) released [Plot](https://observablehq.com/@observablehq/plot) about one day after I got Datasette working. Suddenly, I wanted—nay, needed—to do some fancy visualization with my Stork data so I can really _see_ the stats.

I tweaked my Datasette Dockerfile to install a [token-based authentication plugin](https://github.com/simonw/datasette-auth-tokens). Now my data is secure, but accessible via an API. I wrote an Observable notebook that fetches the most recent usage data and plots it. Now I have a usage dashboard with a graph showing how many hits `stork.js` got per day this year.

It's got a rolling average!

<figure>
{% image "stork-stats.png" %}
<figcaption>I don't think I'm ready to share the y-axis here, sorry.</figcaption>
</figure>

---

I reckon this is the most stable thing I've launched in a prod environment. This feels like the deployment with the smallest bundle of hacks I've ever created. I spent shockingly little time installing software on the EC2 box, which is a task I've easily burned weeks on before. Moreover, I can run each service independently in production or on my personal computer. I'm happy with this because I don't think I'll have to keep worrying about it forever.

Acknowledgements: [Terin](https://twitter.com/terinjokes) helped reaffirm that I was using Docker correctly. Thanks, Terin!
