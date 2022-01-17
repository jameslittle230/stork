---
title:  "Learning and Making"
date:   2018-06-24
layout: post
tags: post
blurb: "What kinds of personal projects help you learn best? When can they stall learning? When should you build in order to learn, and when should you build in order to have something built at the end?"
---

I’ve been working on a crossword puzzle creator for about three years, and I can’t tell whether it’s close to done or barely even started. 
In that time, the project has morphed into a personal playground in which I experiment with new, experimental web technologies. 
In practical terms, this means I’ve been restarting the project fairly regularly, using a new web framework each time.
I've learned how to solve the same problem five different ways.
I've rewritten entire UIs as my design chops have improved.
I've been learning, but I haven't made any progress.[^1]

When I realized that this cyclical development was getting problematic, I stepped back and looked at what I was doing.
This project had become solely devoted to my own learning: I wasn't building a crossword puzzle anymore, I was using the application design for my own education.
My having fallen into this hole wasn't necessarily a bad thing; in fact, there are benefits to experimenting with unknown technologies using a known problem domain. 
The question, therefore, becomes: _which project ideas are better suited as vehicles for learning, and which are better suited for working to completion?_ 

<!--more-->

I’ve found that the best tech playgrounds — these venues for experimenting with new tools, concepts, and techniques — are exciting projects with limited (but flexible) scope, that come from ideas you haven’t put any stake in. 
In other words, productive programming involves separating the learning from the making as much as possible.

## It's easy to get mixed up

Following that advice is hard: my own relationship with writing code blends the learning and making together such that I’m often learning a language while working a new project idea. 
Motivation to write code often only comes when I’m excited about a project: building my crossword app has only happened because I have a “_creating_” itch I want to scratch. 
With that motivation comes grandiose ideas: _”What if the whole app used GraphQL?”_ 
And thus I find myself struggling with two challenges—implementing a GraphQL server and modeling a crossword puzzle word object—simultaneously.

There’s only so much time to write code, and doing only the learning or only the making seems redundant or, hate to say it, even boring.
This lack of excitement goads me to take on as much as possible in a project.
Sometimes still, I’ll get caught up in finding the technology best suited for my project; I’ll explore one until I reach a framework limitation, scrap what I have, and start over using another framework. 
This gives me the repeated short-term dopamine rush of figuring out a new puzzle while simultaneously ensuring that I never get past more than the first leg of what I set out to do.

## Separation of Concerns

With slightly less excitement—by not taking on too many challenges at once—comes more efficiency and a greater ability to follow through on my initial ideas.
I’ve watched this play out in my recent work.
The projects I’ve completed recently use technologies with which I was already familiar; the projects that have been stagnating began as overambitious moonshots.
When I use a familiar, tried-and-true stack, I don’t find myself struggling to implement a solution after I solve a problem in my head. 
Instead, the solutions that come to mind are already modeled using the tools I know.
There aren’t as many dopamine rushes, but instead of puzzling over how to write code, I’m able to crunch through the project’s challenges instead.

I have learned to recognize two different mentalities I take on when I’m writing code. 
The first is one of productivity, in which I’m driven to make something, either by completing or working on a project. 
The second is one in which I want to scratch a curiosity itch, either by reading about what other people are making, or by investigating whether there is a better way to solve a problem. 
However, I often have trouble understanding a framework (or language or concept) if I’m just reading about it; I usually need to write code in order to get a sense of what’s going on. 
When I’m feeling curious, I let myself explore freely.
I might get some reading done, watch some videos, and then either start a new project or branch off of an existing one and play around with the code 
I have no expectation that anything I write will stick. 
If the code I write is significant, I’ll continue down that path; oftentimes though, I’ll take what I learned and incorporate it into something later. 
Regardless of the outcome, I have made sure that block of time is set aside for learning, and I use it only to learn, to experiment, to mess around, knowing that when I’m in the mood to create, the familiar project is right where I left it.

## Ending Thoughts

This system isn’t perfect, but I’ve found it helps to split up my time by examining what I want to do with it. 
When I want to build something, I do so in a way that maximizes efficiency: I use the tools I know to build something I wouldn’t necessarily be able to otherwise. 
When I want to learn, I let my time become unstructured, full of reading, experimenting, and testing.
Sometimes the two blend into each other: a lot of code rewrites have come from learning a better way to solve a problem. 
However, the familiarity that surrounds me while I’m working on a project lets me focus on the problems I want to solve, instead of having an unknown language, framework, or system create more problems at the start.

As I work on personal projects in the future, I'll try to think about the goal of the project: am I building this to create, or am I building this to learn? 
Regardless of the decision—since there is time for both—I'll work with the goal of learning or making as exclusively as possible.

[^1]: Which leads to fun problems such as _”I know I wrote a JSON puzzle parser at some point, but where did it go?”_