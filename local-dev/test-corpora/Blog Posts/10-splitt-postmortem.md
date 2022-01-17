---
title:  "A Bummer of a Postmortem"
date:   2020-01-03
layout: post
tags: post
blurb: "I recently sunsetted a project because the thing already existed. If I never wrote about it, it would be lost to the endless void of time, since I never talked about it anywhere else."
---

I was working on a project called [Splitt](https://splitt.xyz), which was meant to be an interface for figuring out, for a given group of people, who owes money to whom. One person could pay for something, add it to Splitt, and then over time Splitt would reconcile the transactions between the people in the group. My girlfriend and I had a complicated spreadsheet going to do this, and I wanted to build something that did the same thing with a nicer interface and a better data model. I eventually wanted to publicize it, give it a nagware business model like Sublime Text, and have it become a Successful Side Project™. Today, I'm killing it and wanted to write about why.

<!--more-->

I thought it would be a good way to get practice _making something_: particularly something that has a robust UI, a solid REST API that could be consumed by the two clients (Vue on web, and a never-really-finished iOS app), and a good backend data structure. I thought of the idea in the summer (in the shower, where all good ideas originate) and started working on it a few days later. It'd be a good idea, I remember thinking, to get some backend practice in before [starting at Stripe](https://jameslittle.me/blog/2019/next).

I never really checked to see if there was anything like this out there already. When I had something working and showed it to my coworkers, they were very curious about how it compared to [Splitwise](https://www.splitwise.com/), an app I hadn't heard of but probably should have. Turns out Splitwise is basically everything I had wanted Splitt to become—if I had seen Splitt's roadmap through, there wouldn't have been much functional difference between it and Splitwise.

I then had the choice to either keep Splitt going and rethink its roadmap or shut it down and just use Splitwise instead. My girlfriend and I chose to create a Splitwise group (we liked their UI and their functional iOS app better), so Splitt is officially no more. I will eventually be setting Splitt to read-only, and have set the the three repositories (Laravel backend, Vue frontend, and iOS app) to public, so you can examine and criticize my work:

- <https://github.com/jameslittle230/splitt-back>
- <https://github.com/jameslittle230/splitt-vue-front>
- <https://github.com/jameslittle230/splitt-ios>

Ultimately, the only real lesson I can think of here is "make sure you know the competition and the landscape before starting out on a project." If I had known Splitwise already existed, I could have spent more time either working on ways Splitt could have been different, or I could have spent more time working on something else entirely.

I also learned how _difficult_ software can be! I had come up with a lot of infrastructure-y ideas for Splitt, like blue-green deploys, an admin dashboard, and a Stripe integration. But it turned out that building the actual software had to come first by necessity, and that took up enough time that I never got around to all the "bonus" stuff.

I'm still working on side projects—more word coming soon on what I've been doing lately.
