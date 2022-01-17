---
title:  "Developing and Deploying bowdoinorient.com"
date:   2018-09-14
layout: post
tags: post
blurb: "An outline of the system I built to spin up temporary, easily-hacked-on versions of bowdoinorient.com"
---

I moved the [Bowdoin Orient site](http://bowdoinorient.com) from a custom CMS to
a WordPress-based system over a year ago; since that time, I’ve struggled to
find a proper development and deployment workflow that made sense for the
Orient. This year, as the Orient welcomes four new members onto its web staff,
I’ve been working to ensure that the website could easily be developed and
deployed.

The system I built was based on solving the core problem of WordPress
development: that the disadvantages of using exclusively local or remote
development outweigh the simplicity of those two methods. I wanted to ensure
that we could avoid those disadvantages and fulfill other requirements while
using the best best practices I know of. Furthermore, I wrote it in Ruby to
familiarize myself with the language used most often at
[Stripe](https://stripe.com), where I interned this past summer.

<!--more-->

## The problem

WordPress makes it notoriously hard to set up staging environments. There’s
configuration that takes place in PHP files and other configuration that takes
place in the application’s database. Migrating the application from one server
to another is so difficult that paid tools exist to do it properly. Local
development either requires manually configuring daemons and local servers or
installing MAMP. I wanted to build a solution that required as little local
modification as possible, while still providing as much access to the code as
possible.

## More Project Requirements

- **It should be easy to write new code.** I don’t want the development or
  deployment system to get in the way of features that need to be written; in
  addition, I don’t want most of the code I write to be part of the deployment
  system. It should be simple to use and simple to maintain.
- **Servers shouldn’t need ongoing maintenance.** I’m going to graduate soon.
  Whoever works on the Bowdoin Orient site next won’t be interested in the same
  things I’m interested in. Development on the site needs to be able to happen
  even if nobody is massaging the deployment system into place.
- **The workflow should encourage code review and transparency.** One deployment
  system is to have *no* deployment system, and just write all code on the
  server, live. This is known as “cowboy coding,” which is canonically
  [bad](https://en.wikipedia.org/wiki/Cowboy_coding#Disadvantages).

    The system should make it easy to write new code, but it should make it
    *drastically difficult* to write that new code on the actual web server. We
    use Git and Github for version control, and I wanted to ensure that an
    administrator could approve code changes before they made it into the live
    site.

## Methodology

The foundation of the project is a remote web server and database that hosts
different instances of the Orient website. Also running on this server is a
program that can automatically spin up and tear down new instances of the site
and show users how to synchronize code between their local machine and the
server.

When a new instance is created, the Running on a VPS is a Ruby application that
keeps track of development environments, or <em>devenv</em>s. When a new _devenv_ is
made, the user specifies a subdomain, and the application performs a series of
setup steps:

1. It downloads the master branch of the repository into a new folder,
accessible by Apache.
2. It gets the latest database backup, exported by a cron job.
3. It finds and replaces the original url (`bowdoinorient.com`) at which the new
domain that the staging site will be available
(`{something}.test.bowdoinorient.co`).
4. It creates a new MySQL database and a new MySQL user.
5. It imports the modified database backup into that new staging database.
6. It writes `wp-config.php` and `.htaccess` files with accurate database and
domain information.

When this is complete, users use `scp` to copy the entire directory onto their
local computer. They can make edits locally, create new Git branches, and make
commits. As files are changed, developers can use `rsync` to synchronize local
changes with the server. Developers also have full access to the MySQL database.

This lets developers have all the benefits of space on a VPS—server
infrastructure is managed externally, nobody has to do any weird things with
their hosts file to get domains to work, and the site is available
everywhere—while also having all the benefits of local development—they can use
whichever editor feels comfortable, run preprocessing steps locally, and can
have a quick refresh-to-analyze cycle. This is the most optimal system I’ve
found for setting up a hybridized local-remote development.

## Future Work

* This whole project ended up sounding a lot like containerization; I’d love to
  get more familiar with containerization tools to do this even better.
* I took security mildly into account, but there are still some holes I’d like
  to fix.
* I’ve seen some systems that use sockets connected to the running setup scripts
  to give real-time progress updates, instead of relying on a binary HTTP
  response. I’ve worked with sockets [before](https://penguinegg.com), so I knew
  that for the first version of the project they would be complicated enough
  that I wanted to avoid them.
