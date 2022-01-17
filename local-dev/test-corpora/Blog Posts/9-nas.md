---
title: "How I Use my Network-Attached Storage"
date: 2019-10-21
layout: post
tags: post
blurb: "What do I use my Synology DS418j for? How did I get it all set up? What's running on it right now? Let's talk about Plex, Time Machine, file hoarding, and (for some reason) the Pi-Hole."
---

I got, as a gift a few years back, a [Synology DiskStation DS418j](https://www.synology.com/en-us/products/DS418j) along with three Western Digital 8TB Red hard drives. This allowed me to put together a NAS (Network-Attached Storage): a relatively low-powered storage server that lives on my local network. Now that I've moved into an apartment, I've been able to tweak my setup to something relatively enjoyable.

<figure>
{% image "synology.jpg" %}
<figcaption>I dusted it to take the photo, and the photo <em>still</em> looks like trash.</figcaption>
</figure>

# Prelude: Internet in General

I live in an area that's serviced by [Sonic](https://sonic.com), and I'm lucky to get Gigabit internet (roughly 1,000 Mb/s upload and download) coming into my home. That goes into a small, nondescript 8-port Cisco switch that I borrowed a long time ago. That switch goes to a [Google Wifi unit](https://en.wikipedia.org/wiki/Google_Wifi): I have two Google Wifis in the apartment, one which acts as a hub and one which acts as a repeater. The switch also sends wires to the Synology, a Raspberry Pi, and my computer.

If I had the choice to re-do things, I wouldn't get the Google Wifi unit for multiple reasons, the first of which being that the system doesn't work with Synology's DDNS: either the port-forwarding gets blocked, or the DNS resolution does, I haven't figured out which. The eventual goal is to switch to a more configurable system—I hear [Ubiquiti](https://unifi-network.ui.com) makes a good system, but I haven't done enough research to say I'll commit one way or another.

# The Synology

I primarily use the Synology as file storage. When you set up the operating system, you can decide how you want the RAID setup to work. I think the best option (and also the default option) is SHR, which is similar to RAID 5 but works with different size disks.[^0] I set up a single volume and have multiple directories in that volume's root.

The first directory in root is called "Files", and it stores files. I keep all the documents I created for school, papers I found online, fonts, things like a PDF of my lease, backups of my photo libraries (I'll get back to that), etc. Anything I don't think I'll need immediately gets offloaded from my computer hard drive to the Files directory: this keeps my computer's boot drive nicely not-full. I was also able to stop using Dropbox by storing my documents on my Synology: I was only really using my Synology as a way of getting to my files from elsewhere; Synology's remote login has eliminated this. Files is generally uncomplicated, I mostly use it as a loosely-sorted write-only archive of my digital life.

Importantly, I _don't_ store things like movies in my "Files" directory. I set up my Synology as a Plex server, and so my media lives elsewhere. Plex is neat: I can store video and audio files on the Synology, install a server application on the Synology, and then install client applications elsewhere (like the web, or on an Apple TV) and I can stream media from my Plex to that client application. I used [a tutorial from 9to5Mac](https://9to5mac.com/2019/07/26/set-up-plex-synology-nas/) to set up Plex on my Synology. Importantly, which either this tutorial leaves out or I overlooked, the Plex installer will set up a new Plex user and will expect media to live in `/Homes/Plex`[^1], which in turn assumes that each user on the Synology has a home directory. It's a bit weird for each user to have a home directory _and_ have them all using the Files directory in the volume's root, but here we are.

I also use the Synology as a Time Machine destination for my computer and my girlfriend's computer. I used [another tutorial](https://nascompares.com/2019/04/08/how-to-back-up-your-mac-to-synology-nas-with-time-machine/) for getting that working, and I've just been able to set it and forget it -- it's apparently (if my reported Time Machine stats are accurate) been chugging along for months now just fine. Importantly, though, that means I have two more users and two more directories in root: "JL Time Machine" for my backups and "MT Time Machine" for my girlfriend's.

All these files are backed up to Amazon Glacier via a twice-per-week cron job, set up through the Glacier Backup package. Glacier [might not be the best tool to back up all my files](https://medium.com/@karppinen/how-i-ended-up-paying-150-for-a-single-60gb-download-from-amazon-glacier-6cb77b288c3e), but it is not meant to be my primary offsite backup forever: I'll probably opt to start giving Backblaze some money soon and also back stuff up to their B2 service. For now, though, this Glacier setup _really_ gives me peace-of-mind. I've lost some videos of my childhood to the unending churn of hard drives, and I like knowing that once I put something on my Synology I'll have it ~forever, unless I really try hard to delete it or someone nukes us-east-1 or something.

I mostly access the Synology through the web interface (for browsing files, changing settings, and installing packages), through the command line for running scripts, or through [Transmit](https://www.panic.com/transmit/), which is a gorgeous file transfer application made by Panic. That means I need to make the web interface ports and the SSH ports available—SSH is used both for logging into the box in the Terminal and for SFTP, which is how I connect Transmit to the Synology.[^2] I have QuickConnect set up, which works fine: it proxies your Synology to the outside world via proprietary magic. I also had Synology's DDNS set up, which worked much better than QuickConnect but broke when I started using Google Wifi as my router, since Google Wifi and DDNS don't work together. I don't enable SSH on the standard port, but it's moot anyway since there isn't a public IP address that resolves to the Synology.

# An Interlude: The Pi-Hole

A month ago I bought a Raspberry Pi 3 model B on Amazon just for fun. Eventually I turned it into a Pi-hole: a DNS server that drops hostnames that are known advertising hosts, effectively giving myself a network-wide ad blocker. I used [a blocklist I found on Reddit](https://www.reddit.com/r/pihole/comments/bppug1/introducing_the/) and it's been outstanding -- I can't recommend it enough. I've gotten to the point where I go to work and am bummed that the internet there is jankier because of all the ads. I get cool stats. I save bandwidth. What's not to love?

(Admittedly, this isn't related to my NAS [other than the fact that they're the only two devices on my network that have reserved IPs] but it _is_ part of my networking setup, and I felt like it was important to include.)

# Conclusion?

I think my Synology setup is very good. I like the file storage. I like that I can throw big [`youtube-dl`](https://youtube-dl.org) jobs on it and it'll just churn and download everything in the background. I like that I can keep every single photo I take in RAW format until the end of time -- I don't have to be judicial with how I spend my data. I like that when my hard drive is getting full, I can just drag and drop a lot of files to the little box that lives under my desk and my hard drive will be no longer full.

It's good. It's worth it, I'd say.

<!-- Footnotes -->

[^0]: Raid 5 stores unique data on n-1 disks, then sets the last disk to be the XOR of all the previous disks. SHR does the same thing, but makes sure the XOR'd disk is of the biggest volume (so the XOR data will fit).
[^1]: Plex has a very opinionated way it wants you to organize your media library. I don't have the time to get into it now, but needless to say it not only matters where the media lives but also how it's curated.
[^2]: I also have whatever ports are used for the Time Machine file sharing system: maybe AFP, or maybe Bonjour? I have to reiterate how much it was a "set it up and never think about it again" kind of operation—I'm not really sure what it's doing.
