/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *  http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */
//!  Poll the GPS sensor every 10 seconds. Transmit the sensor data to the CoAP server after polling.
//!  This is the Rust version of https://github.com/lupyuen/stm32bluepill-mynewt-sensor/blob/rust-nbiot/apps/my_sensor_app/OLDsrc/gps_sensor.c

use mynewt::{
    result::*,                              //  Import Mynewt API Result and Error types
    hw::sensor_mgr,                         //  Import Mynewt Sensor Manager API
    hw::sensor::{        
        self,                               //  Import Mynewt Sensor API
        sensor_type_t,
    },
    sys::console,                           //  Import Mynewt Console API
    Strn,                                   //  Import Mynewt macros    
};
use mynewt_macros::{ init_strn };           //  Import Mynewt procedural macros
use crate::app_network;                     //  Import `app_network.rs` for sending sensor data

///  Sensor to be polled: `gps_l70r_0` is the Quectel L70-R GPS module
static GPS_DEVICE: Strn      = init_strn!("gps_l70r_0");
///  Poll GPS every 11,000 milliseconds (11 seconds)  
const GPS_POLL_TIME: u32     = (11 * 1000);  
///  Use key (field name) `geo` to transmit GPS geolocation to CoAP Server
const GPS_SENSOR_KEY: Strn   = init_strn!("geo");
///  Type of sensor: Geolocation (latitude, longitude, altitude)
const GPS_SENSOR_TYPE: sensor_type_t = sensor::SENSOR_TYPE_GEOLOCATION;

///  Ask Mynewt to poll the GPS sensor and call `aggregate_sensor_data()`
///  Return `Ok()` if successful, else return `Err()` with `MynewtError` error code inside.
pub fn start_gps_listener() -> MynewtResult<()>  {  //  Returns an error code upon error.
    //  Start the GPS driver.
    console::print("Rust GPS poll\n");
    start_gps_l70r() ? ;

    //  Fetch the sensor by name.
    let sensor = sensor_mgr::find_bydevname(&GPS_DEVICE)
        .next()             //  Fetch the first sensor that matches
        .expect("no GPS");  //  Stop if no sensor found

    //  At power on, we ask Mynewt to poll our GPS sensor every 11 seconds.
    sensor::set_poll_rate_ms(&GPS_DEVICE, GPS_POLL_TIME) ? ;

    // Create a sensor listener that will call function `aggregate_sensor_data` after polling the sensor data
    let listener = sensor::new_sensor_listener(
        &GPS_SENSOR_KEY,  //  Transmit as field: `geo`
        GPS_SENSOR_TYPE,  //  Type of sensor data: GPS Geolocation
        app_network::aggregate_sensor_data  //  Call this function with the polled data: `aggregate_sensor_data`
    ) ? ;

    //  Register the Listener Function to be called with the polled sensor data.
    sensor::register_listener(sensor, listener) ? ;  //  `?` means in case of error, return error now.

    //  Return `Ok()` to indicate success.  This line should not end with a semicolon (;).
    Ok(())
}

/// Start the GPS driver for Quectel L70R
fn start_gps_l70r() -> MynewtResult<()> {
    //  Driver function to start the GPS
    extern { fn gps_l70r_start() -> i32; }
    let res = unsafe { gps_l70r_start() };
    if res == 0 { Ok(()) }
    else { Err(MynewtError::from(res)) }
}