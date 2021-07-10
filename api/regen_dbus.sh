#!/usr/bin/env bash

dbus-codegen-rust -s --file dbus/introspection.xml -c nonblock -p / -m none -i net.connman.iwd >src/services/iwd/dbus_iwd.rs
