---
title:  "About the Guestbook"
date: 2020-03-16
layout: post
tags: post
blurb: "It's interesting to think that a feature that seems so intuitively simple can have so many challenges, questions, and choices once you start building it. Here are the ones I faced while building /guestbook."
---

Adding the [guestbook](/guestbook) to my site was really fun: it was the kind of project that seemed easy at first, but had a lot of complexity under the hood. I kept thinking I was done, then realizing there was some edge case I hadn't quite thought of. I wanted to write down some of those edge cases and subtle challenges I faced, mostly to show that features that seem simple on the outset might actually take a while to put together.

Some notes: during the course of development, I decided to optimize for _cost_ first, then _speed_, then _safety_, then _availability_ in that order.

- The guestbook is two parts: a form and a list of entries. I need to store those entries somewhere, and I need to be able to fetch them on demand. I could have used a MySQL database and used a server-side scripting language to build the page. That would probably take a VPS, which would cost $5/month. Instead, I built up a REST API using AWS Lambda as the application layer and S3 as the storage layer.
- Given that model, I had to figure out how I would read from and write to my S3 data store. I decided that creating a new entry would write a new file to S3 and getting all the entries would loop through all the files in the bucket, get their contents, and output a list. (This also means that reading is likely more expensive than writing, since it uses O(n) S3 reads for every GET request.)
- I wanted to make sure nobody spams my API. I'm using AWS' API Gateway [Usage Plans](https://docs.aws.amazon.com/apigateway/latest/developerguide/api-gateway-api-usage-plans.html). Ultimately I want to make sure I'm letting as many people legitimately make API requests as possible without overrunning my Lambda budget or my S3 budget.
- I needed to figure out which endpoints my API had. People need to read data and write data, so that sounds like a POST and a GET.
  - Will people need to read data in different ways, like getting individual entries?
  - Will people need to perform other actions, like message deletion?
  - The answer to both of these questions might be "yes" in the future, but for now I'm passing on both of them.
- I had to define what a valid request looks like, for both my POST endpoint and my GET endpoint.
- I had to figure out how to validate the requests, and write up a system for doing so. As part of this, I had to figure out which input parameters are required and which are optional, and for each parameter, what "validity" looks like.
- I had to figure out what my validation errors look like. What happens if you don't include a required param? What happens if you include an unexpected param? I decided that since I was controlling the frontend and backend, I could be as strict as I wanted and throw errors judiciously (instead of maybe dropping unknown fields).
- I had to define what a valid response looks like, for both endpoints, in the success and failure cases.
- I had to set up CORS headers in API Gateway.
- I also had to make sure that Lambda errors were translating into API Gateway 400s correctly.
- I had to write tests for my API to make sure all the inputs I thought of got to their respective outputs.
- Eventually I also ended up setting up a QA database and writing a little hook so I could avoid writing to my prod database while I'm developing. This ended up being nice in theory but unused in practice.
- This is all user-submitted data. How do I get notified that a new entry has been submitted, and do I even want those notifications? How can I make sure nothing gross is being put up on my page? How do I block users that are abusing the guestbook? (These are all unsolved problems... for now.)

The frontend was, shockingly, even more challenging.

- I had to figure out how often the client would fetch the data (i.e. hit the GET endpoint). I assumed once per page load, but wondered if more was necessary.
- I then wondered if that assumption even true? Instead, I could cache data on page loads and show you cached data instead of running the lambda function on every page load. (Ultimately I didn't do this.)
- I had to figure out my UI state. I realized there were two independent state machines: the success/loading/failure when fetching the guestbook entries, and the success/loading/failure when submitting the form.
- I had to define UI for each of those states.
- I wanted a message at the top of the list that said something like "3 messages." I had to make sure "messages" wasn't pluralized when there was one message in the list.
- I had to figure out what happens when you submit the form. I considered adding client-side validation, but decided that would be too complex for a feature I had already built into the backend.
- If you submit the form successfully, the form values should disappear, and we should show a success message.
- I had to decide whether or not to re-fetch the data if the form was submitted successfully. Ultimately I decided it shouldn't; it should "fake it" by adding the entry to the local store of entry objects. This would reduce the number of GET calls, and would be a valid tradeoff based on what the writer cares about: they don't need to see all the entries that were added between page load and form submission, they just want to see their entry added as part of the greater list.
- If the form doesn't validate, display the error and keep all existing data present.
- I put a lot of thought into dates here, both how they're stored and how they're displayed. Ultimately, I decided that the Lambda function would determine the creation timestamp of a given entry, which meant the value _wasn't_ present on the submitted object when the client created it and sent it to the server. However, the date field has to have something in it when it gets appended to the displayed list of entries. Ultimately I ended up appending a "date" value to the submitted object before adding it to the local store; this date value is likely different from what the server records, but only by a factor of milliseconds.
- I still have to figure out if date display works in other timezones besides PST.
- I had to determine when to display relative dates ("two days ago") vs. absolute dates (2020-03-14). I wanted to provide a hook so that if you want to know the absolute date, you could see it even if the relative date was visible.

I was very surprised that "the page should record entries from this form and display them on the bottom of the page" turned into a week-long project in which I had to grapple with all these challenges!

If you haven't yet, go [write something in the guestbook](/guestbook), if for no other reason than so I don't look silly.