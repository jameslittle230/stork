---
title: "How to train the Tensorflow Object Detection API with custom training data"
date: 2019-01-25
popular: true
layout: post
tags: post
blurb: "A guide to setting up a Tensorflow Object Detection system and training it with your own self-annotated data."
---

I’ve been working on image object detection for [my senior thesis](https://honors.jameslittle.me/) at [Bowdoin](https://www.bowdoin.edu/computer-science/index.html) and have been unable to find a tutorial that describes, at a low enough level (i.e. with code samples), how to set up the [Tensorflow Object Detection API](https://github.com/tensorflow/models/tree/master/research/object_detection) and train a model with a custom dataset. This aims to be that tutorial: the one I wish I could have found three months ago.

<!--more-->

## Background

I don’t know much about advanced Machine Learning concepts, and I know even less about Tensorflow. If you’re like me, you’ve heard of Tensorflow as the best Machine Learning framework available today[^1], and you want to use it for a very specific use case (in my case, object detection). You also don’t want to spend months studying what seem to be Tensorflow-specific vocabulary and concepts.

Similarly, if you’re like me, you have some familiarity with Linux and Python. I’m a programmer more than I am an ML researcher, and you’ll probably grok this article most if you’re in the same boat.

Finally, I assume you have Tensorflow installed—at the very least, I assume you can run the “getting started” block of code on the [Tensorflow Tutorial page](https://www.tensorflow.org/tutorials/). If that runs without any errors, you should be good to go. If not, this might not be the tutorial for you, and you should get that working before coming back here.

## The Plan

Since this is a complicated process, there are a few steps I’ll take you through (and therefore, a few sections this article will be broken into):

1. Set up the Object Detection API
2. Get your datasets (both training and testing) in a format Tensorflow can understand
3. Train and test the API with those datasets

Finally, during the training step, we’ll set up [TensorBoard](https://www.tensorflow.org/guide/summaries_and_tensorboard), a browser-based training visualization tool, to watch our training job over time. Using this model in a different environment (like a mobile device) is, unfortunately, beyond the scope of this article.

## Step One: Set up the Object Detection API

This section will lead you through four steps:

- Download the Object Detection API’s code and copy the relevant parts into a new subdirectory, `my_project`
- Install and compile Protocol Buffers
- Install and build the python modules necessary for the API
- Test that the API is ready for use

You’ll need a new directory for all of our future work steps, so start by creating one and changing into it.

```bash
$ mkdir obj_detection

$ cd obj_detection
```

The Object Detection API is part of a [large, official repository](https://github.com/tensorflow/models/graphs/contributors) that contains lots of different Tensorflow models. We only want one of the models available, but we’ll download the entire Models repository since there are a few other configuration files we’ll want.

```bash
$ git clone https://github.com/tensorflow/models.git
```

Once that download is over, we’ll copy the files into a new directory.

```bash
$ mkdir my_project

$ cp -r models/research/object_detection my_project

$ cp -r models/research/slim my_project

$ cp models/research/setup.py my_project
```

The Object Detection API uses [Protocol Buffers](https://developers.google.com/protocol-buffers/) (Protobufs), a message serialization and transmission framework, for some reason I’m not entirely sure about. We need to download and compile Protobufs, however, to get the API to work.

Downloading and unzipping Protobufs will create a `bin` directory in your `obj_detection` directory.

```bash
$ wget -O protobuf.zip https://github.com/google/protobuf/releases/download/v3.0.0/protoc-3.0.0-linux-x86_64.zip

$ unzip protobuf.zip
```

At this point, you’ll need to compile the protobufs. You need to move into the `my_project` directory and run the compilation script from there, since there are import steps that make assumptions about the location from which you’re running the script.

```bash
$ cd my_project

$ ../bin/protoc object_detection/protos/*.proto --python_out=.
```

With Protobufs downloaded and compiled, the Object Detection Python module has to be built. This will create a `build` directory in your `my_project` directory.

The following commands should be run from your `my_project` directory: the place you `cd`-ed into in the last step.

```bash
$ export PYTHONPATH=$(pwd):$(pwd)/slim/:$(pwd)/lib/python3.4/site-packages/:$PYTHONPATH

$ python3 setup.py install --prefix $(pwd) # Lots of output!

$ python3 setup.py build
```

These commands will first designate the current directory (and some subdirectories) as locations Python is allowed to read modules from and write modules to. The second command will install all the various Python modules necessary for the API to the current directory, and the last command will build those modules.

When you’re done, you’ll have a directory structure that looks like this:

```bash
$ ls # From obj_detection
bin  include  models  my_project  protobuf.zip  readme.txt

$ cd my_project

$ ls
bin  build  dist  lib  object_detection  object_detection.egg-info  setup.py  slim
```

Finally, the following command will test your setup: if you get an `OK` at the end, you’re good to go.

```bash
# From my_project

$ python3 object_detection/builders/model_builder_test.py
```

### Troubleshooting

You might _not_ get an `OK` at the end. Unfortunately, I can’t troubleshoot your setup for you, but I can tell you that when I was troubleshooting my own, the most finicky part I encountered was setting the `PYTHONPATH` correctly; we did this above with the line that began with `export`.

StackOverflow actually works pretty well to figure out what other things you should put in your `PYTHONPATH` I had an issue finding `libcublas.so.9.0`; for me, that meant running `$ export PYTHONPATH=/usr/local/cuda-9.0/lib64/:$PYTHONPATH`.

If you leave the project for a while and come back, you’ll have to run all the `export` lines again to restore your `PYTHONPATH`. I created a bash file that ran all the necessary `export` commands, and I would run `$ source setup_python_path.sh` whenever I logged on.

For completion’s sake, I also include the line `export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/usr/local/cuda-9.0/targets/x86_64-linux/lib/` in my script; running the test script wouldn’t work without it. This is probably relevant only to my computer, but it might help you out too.

## Step Two: Preparing the Datasets

Since the API we’re using is based on object detection, you’ll need to have a dataset you want to work with. This dataset should be comprised of images and annotations in whatever format you choose: I had JPGs numbered `0.jpg` through `9999.jpg` and a CSV file with the coordinates of the objects I’m detecting.[^2]

{% image "files.png" %}

For each object in an image, you should have `x1`, `x2`, `y1` and `y2` coordinates available, where `(x1, y1)` is the upper left corner of the rectangle and `(x2, y2)` is the lower right corner of the rectangle.

{% image "box.png" %}

You’ll probably have two of these datasets, one large one for training and one smaller one for testing. We’ll be taking the two datasets and transforming each of them into [`.tfrecord` files:](https://medium.com/mostly-ai/tensorflow-records-what-they-are-and-how-to-use-them-c46bc4bbb564) large binary files that contain a complete representation of the entire dataset.

I wrote a Python script to do this. The script reads in each image in a directory, reads the corresponding line in a CSV file, and appends the TFRecord with the image data and the associated coordinate data. Your script will probably look different since this is based on my dataset and this will be based on yours.

Remember: If you’ve logged out of your shell since setting up your Python path, you’ll have to set it up again before running this script.

```python
import tensorflow as tf
from object_detection.utils import dataset_util

flags = tf.app.flags

# Here is where the output filename of the TFRecord is determined. Change this,
# perhaps to either `training.tfrecord` or `testing.tfrecord`.
flags.DEFINE_string('output_path', 'output.tfrecord', 'Path to output TFRecord')
FLAGS = flags.FLAGS


def create_tfrecord(filename, coords):
    # You can read these in from your image, or you can hack it and
    # hardcode the dimensions in.
    height = 480
    width = 640

    filename = str.encode(filename)

    with open(filename, 'rb') as myfile:
        encoded_image_data = myfile.read()

    image_format = b'jpeg'  # b'jpeg' or b'png'

    xmins = [coords[0] / width]
    xmaxs = [coords[1] / width]
    ymins = [coords[2] / height]
    ymaxs = [coords[3] / height]

    # Here, you define the "classes" you're detecting. This setup assumes one
    # class, named "Ball". Your setup will probably look different, so be sure
    # to change these lines.
    classes_text = [b'Ball']
    classes = [1]

    tfrecord = tf.train.Example(features=tf.train.Features(feature={
        'image/height': dataset_util.int64_feature(height),
        'image/width': dataset_util.int64_feature(width),
        'image/filename': dataset_util.bytes_feature(filename),
        'image/source_id': dataset_util.bytes_feature(filename),
        'image/encoded': dataset_util.bytes_feature(encoded_image_data),
        'image/format': dataset_util.bytes_feature(image_format),
        'image/object/bbox/xmin': dataset_util.float_list_feature(xmins),
        'image/object/bbox/xmax': dataset_util.float_list_feature(xmaxs),
        'image/object/bbox/ymin': dataset_util.float_list_feature(ymins),
        'image/object/bbox/ymax': dataset_util.float_list_feature(ymaxs),
        'image/object/class/text': dataset_util.bytes_list_feature(classes_text),
        'image/object/class/label': dataset_util.int64_list_feature(classes),
    }))
    return tfrecord


def main(_):
    writer = tf.python_io.TFRecordWriter(FLAGS.output_path)

    with open("annotations.csv") as fp:
        line = fp.readline()
        while line:
            data = line.split(",")
            tfrecord = create_tfrecord("img/out/{}.jpg".format(data[0]), data[1:])
            writer.write(tfrecord.SerializeToString())
            line = fp.readline()
        writer.close()

  print("Done.")


if __name__ == '__main__':
    tf.app.run()
```

Make sure you know when your script is done running — a `print()` call should do just fine. When you have your two completed `.tfrecord` files (one for the training dataset and one for the testing dataset), put them somewhere and hold onto them for later.

## Step 3: Training and Testing

While this tutorial describes how to train the Object Detector API using your own data, it isn’t describing how to train a model _from scratch_. This distinction is important: instead of starting from nothing, we’ll be starting with an existing, generalized object detection model and continuing to train it based on our own data. Models, in Tensorflow’s world, can simultaneously be independent entities and checkpoints, meaning that after training a model for a long while, you can either pack up and call it a day and use that model in the wild, _or_ you can stop for a bit and resume training later. We’re doing more of the second option, although instead of resuming the exact same training, we’re nudging an existing model (which I’ll call the _baseline model_) towards the object detection we want it to be able to perform. This lets us get some results fairly quickly — the existing models have been trained on very high powered computers for a very long time, and our tweaks take only a little bit of time.

The Object Detection API provides [a set of these baseline models](https://github.com/tensorflow/models/blob/master/research/object_detection/g3doc/detection_model_zoo.md); they allow you to either use them out of the box or initialize new models based on them. I used “SSD with Inception v2 configuration for MSCOCO Dataset,” but you might want to use a different baseline model depending on what you’re trying to detect. To download the one I used, run the following command:

```bash
$ wget http://download.tensorflow.org/models/object_detection/faster_rcnn_inception_resnet_v2_atrous_oid_14_10_2017.tar.gz

$ tar -zxvf faster_rcnn_inception_resnet_v2_atrous_oid_14_10_2017.tar.gz
```

Training and testing happen at the same time — the scripts in the API run a testing step after every training step. To begin the training/testing, we’ll first need a configuration file; the configuration file framework you use depends on which baseline model you’re using. (If you’re not using a baseline model, you can either write your own or modify one of the given ones.)

The configuration I used is below, and originally comes from [the Object Detection API’s sample configs](https://github.com/tensorflow/models/blob/master/research/object_detection/samples/configs/ssd_inception_v2_coco.config). The file gets saved in your working directory as `mymodel.config` (although the actual filename doesn’t totally matter). I’ve marked within the file the lines you should modify, and kept the original comments as well.

```
model {
  ssd {
    num_classes: 1 # Change this!
    box_coder {
      faster_rcnn_box_coder {
        y_scale: 10.0
        x_scale: 10.0
        height_scale: 5.0
        width_scale: 5.0
      }
    }
    matcher {
      argmax_matcher {
        matched_threshold: 0.5
        unmatched_threshold: 0.5
        ignore_thresholds: false
        negatives_lower_than_unmatched: true
        force_match_for_each_row: true
      }
    }
    similarity_calculator {
      iou_similarity {
      }
    }
    anchor_generator {
      ssd_anchor_generator {
        num_layers: 6
        min_scale: 0.2
        max_scale: 0.95
        aspect_ratios: 1.0
        aspect_ratios: 2.0
        aspect_ratios: 0.5
        aspect_ratios: 3.0
        aspect_ratios: 0.3333
        reduce_boxes_in_lowest_layer: true
      }
    }
    image_resizer {
      fixed_shape_resizer {
        height: 300
        width: 300
      }
    }
    box_predictor {
      convolutional_box_predictor {
        min_depth: 0
        max_depth: 0
        num_layers_before_predictor: 0
        use_dropout: false
        dropout_keep_probability: 0.8
        kernel_size: 3
        box_code_size: 4
        apply_sigmoid_to_scores: false
        conv_hyperparams {
          activation: RELU_6,
          regularizer {
            l2_regularizer {
              weight: 0.00004
            }
          }
          initializer {
            truncated_normal_initializer {
              stddev: 0.03
              mean: 0.0
            }
          }
        }
      }
    }
    feature_extractor {
      type: 'ssd_inception_v2'
      min_depth: 16
      depth_multiplier: 1.0
      conv_hyperparams {
        activation: RELU_6,
        regularizer {
          l2_regularizer {
            weight: 0.00004
          }
        }
        initializer {
          truncated_normal_initializer {
            stddev: 0.03
            mean: 0.0
          }
        }
        batch_norm {
          train: true,
          scale: true,
          center: true,
          decay: 0.9997,
          epsilon: 0.001,
        }
      }
      override_base_feature_extractor_hyperparams: true
    }
    loss {
      classification_loss {
        weighted_sigmoid {
        }
      }
      localization_loss {
        weighted_smooth_l1 {
        }
      }
      hard_example_miner {
        num_hard_examples: 3000
        iou_threshold: 0.99
        loss_type: CLASSIFICATION
        max_negatives_per_positive: 3
        min_negatives_per_image: 0
      }
      classification_weight: 1.0
      localization_weight: 1.0
    }
    normalize_loss_by_num_matches: true
    post_processing {
      batch_non_max_suppression {
        score_threshold: 1e-8
        iou_threshold: 0.6
        max_detections_per_class: 100
        max_total_detections: 100
      }
      score_converter: SIGMOID
    }
  }
}

train_config: {
  batch_size: 24
  optimizer {
    rms_prop_optimizer: {
      learning_rate: {
        exponential_decay_learning_rate {
          initial_learning_rate: 0.004
          decay_steps: 800720
          decay_factor: 0.95
        }
      }
      momentum_optimizer_value: 0.9
      decay: 0.9
      epsilon: 1.0
    }
  }
  fine_tune_checkpoint: "YOUR-BASELINE-MODEL/model.ckpt" # Change this to point to your baseline model -- in the file you just downloaded
  from_detection_checkpoint: true
  # Note: The below line limits the training process to 200K steps, which we
  # empirically found to be sufficient enough to train the pets dataset. This
  # effectively bypasses the learning rate schedule (the learning rate will
  # never decay). Remove the below line to train indefinitely.

  num_steps: 400000
  data_augmentation_options {
    random_horizontal_flip {
    }
  }
  data_augmentation_options {
    ssd_random_crop {
    }
  }
}

train_input_reader: {
  tf_record_input_reader {
    input_path: "YOUR-TRAINING-TFRECORD/training.tfrecord"
  }
  label_map_path: "YOUR-LABELMAP/labelmap.txt"
}

eval_config: {
  num_examples: 8000
  # Note: The below line limits the evaluation process to 10 evaluations.
  # Remove the below line to evaluate indefinitely.
  max_evals: 10
}

eval_input_reader: {
  tf_record_input_reader {
    input_path: "YOUR-TESTING-TFRECORD/testing.tfrecord"
  }
  label_map_path: "SAME-LABELMAP-AS-ABOVE/labelmap.txt"
  shuffle: false
  num_readers: 1
}
```

Finally, you’re ready to run the detector. Put the following into a file called `run.sh`:

```bash
PIPELINE_CONFIG_PATH="YOUR-CONFIG.config"
MODEL_DIR="./object_detection/modeldir"
NUM_TRAIN_STEPS=50000 # Change this if necessary
SAMPLE_1_OF_N_EVAL_EXAMPLES=1

python3 object_detection/model_main.py \
    --pipeline_config_path=${PIPELINE_CONFIG_PATH} \
    --model_dir=${MODEL_DIR} \
    --num_train_steps=${NUM_TRAIN_STEPS} \
    --sample_1_of_n_eval_examples=$SAMPLE_1_OF_N_EVAL_EXAMPLES \
    --alsologtostderr
```

And finally, you can run that script (`$ ./run.sh`) to get the training job started.

### TensorBoard

TensorBoard is a program that comes with Tensorflow that starts up a local web server and hosts a dashboard to show the progress of the training and testing job. You can set up Tensorboard to watch your model directory (`./object_detection/modeldir`) and it can describe the progress of your training job.

In a new terminal (so as to not disturb your training job), navigate back to your `my_project` directory, reconfigure your Python path, and then run:

```bash
$ tensorboard --logdir=./object_detection/modeldir
```

With that running, you can navigate to http://localhost:6006 and watch the graphs go by over the next few hours or days

## Conclusion

I wrote this up because I couldn’t find a tutorial online that went through these same steps, so I hope this ends up being helpful for the next person who wants to embark on this same journey.

I’m also sure I got some things wrong; reach out if I have an error and I’ll work to correct it.

---

## Other Resources

I clearly looked up a lot of ways other people were doing this. Unfortunately, due to the nature of online research, it will be impossible for me to list everything I encountered on the internet that helped me along the way. However, here is an incomplete list of sources (and an implied list of apologies to those who I forgot):

- The [official Object Detection docs](https://github.com/tensorflow/models/tree/master/research/object_detection/g3doc): these articles ([1](https://github.com/tensorflow/models/blob/master/research/object_detection/g3doc/detection_model_zoo.md) [2](https://github.com/tensorflow/models/blob/master/research/object_detection/g3doc/installation.md) [3](https://github.com/tensorflow/models/blob/master/research/object_detection/g3doc/preparing_inputs.md) [4](https://github.com/tensorflow/models/blob/master/research/object_detection/g3doc/running_locally.md) [5](https://github.com/tensorflow/models/blob/master/research/object_detection/g3doc/running_pets.md) [6](https://github.com/tensorflow/models/blob/master/research/object_detection/g3doc/using_your_own_dataset.md) [7](https://github.com/tensorflow/models/blob/master/research/object_detection/g3doc/oid_inference_and_evaluation.md)) were the most helpful.
- This [toy detector tutorial](https://deeplearninganalytics.org/blog/building-toy-detector-with-object-detection-api) by Priya Dwivedi
- [This Medium article](https://medium.com/@WuStangDan/step-by-step-tensorflow-object-detection-api-tutorial-part-1-selecting-a-model-a02b6aabe39e) by Daniel Stang, and [its sequel](https://medium.com/@WuStangDan/step-by-step-tensorflow-object-detection-api-tutorial-part-2-converting-dataset-to-tfrecord-47f24be9248d)
- An [O’Reilly article](https://www.oreilly.com/ideas/object-detection-with-tensorflow) by Justin Francis

[^1]: Don’t @ me
[^2]: Getting this dataset and figuring out how to annotate it is up to you — since we’re dealing with _your_ dataset here, it wouldn’t make sense for me to give you instructions for doing this.
