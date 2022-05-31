let driving = false;
const Start_Button = document.getElementById("drive-toggle");
Start_Button.addEventListener('click', toggleDrive);
const Shift_Button = document.getElementById("mid-drive-toggle");
let coordinates = [];
let startTime;

!function() {
    console.log(innerHeight, innerWidth)
    if (window.innerWidth > window.innerHeight) {
        console.warn("GPS features are not available on PC!");
        Start_Button.remove();
        Shift_Button.remove();
        const textnode = document.createTextNode("GPS features are not available on PC!");
        document.getElementById("map").appendChild(textnode);
        return;
    }

    function init() {
        console.log("Prog1");
        startTime = Date.now();
        Start_Button.hidden = false;

    var map = L.map('map', {
        center: [0,0],
        zoom: 20
    });
    
    setTimeout(function () {   
        if (!map)
        loadmap(); /*load map function after ajax is complitly load */
        }, 1000);

    setTimeout(function () { 
        if (map)
        map.invalidateSize();
        }, 1500);
    
    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
    attribution: '<a href="http://openstreetmap.org/copyright">OpenStreetMap</a> contributors</a>'
    }).addTo(map);
    console.log("Prog2");
    var options = {
    zoom: 20,
    enableHighAccuracy: true,
    timeout: 5000,
    maximumAge: 0
    };
    function error(err) {
    console.warn(`ERROR(${err.code}): ${err.message}`);
    }
    console.log("Prog3");
    L.Control.geocoder().addTo(map);
    if (!navigator.geolocation) {
        console.log("Your browser doesn't support geolocation feature!");
    } else {
        setInterval(() => {
            navigator.geolocation.getCurrentPosition(getPosition, error, options)
        }, 1000);
    };
    var marker, circle, lat, long, accuracy;
    console.log("Prog4");
    function firstPosition(position) {
        console.log(position);
        lat = position.coords.latitude;
        long = position.coords.longitude;
        var marker = L.marker([lat, long]);    // Creating a Marker
        
        // Adding popup to the marker
        marker.bindPopup('Start Location').openPopup();
        marker.addTo(map); // Adding marker to the map
    }
    navigator.geolocation.getCurrentPosition(firstPosition, error, options);

    var myLines, marker;
    function getPosition(position) {
        lat = position.coords.latitude;
        long = position.coords.longitude;
        accuracy = position.coords.accuracy;
        console.log(lat,long);
        if (driving) {
            coordinates.push([long, lat]);
            map.setView([lat, long], 16);
            if (marker) {
                map.removeLayer(marker);
            }
            marker = L.marker([lat, long]);    // Creating a Marker
            
            // Adding popup to the marker
            marker.bindPopup('Current Location').openPopup();
            marker.addTo(map); // Adding marker to the map
        }
        if (!coordinates) {
            return;
        }
        if (myLines) {
            map.removeLayer(myLines);
        }
        

        myLines = [{
            "type": "LineString",
            "coordinates": coordinates
        }];
        
        var myStyle = {
            "color": "#ff7800",
            "weight": 3,
            "opacity": 0.65
        };
        L.geoJSON
        
        L.geoJSON(myLines, {
            style: myStyle
        }).addTo(map);
        

    }
    }
    
    var data = "";
    function err() {
        data = "err";
    }
    console.log("Wello");
    navigator.geolocation.getCurrentPosition(function() {}, err);
    if (data == "err") {
        console.warn("GPS features are not enabled");
        Start_Button.remove();
        Shift_Button.remove();
        const textnode = document.createTextNode("GPS is not enabled");
        document.getElementById("map").appendChild(textnode);
        return;
    }
    console.log("www");
    init();
    /*    
    function handlePermission() {
      navigator.permissions.query({name:'geolocation'}).then(function(result) {
        console.log(result);
        result.addEventListener('change', function() {
            window.location.reload();
          });
        if (result.state == 'granted') {
            console.log("gg");
        } else if (result.state == 'prompt') {
            console.log("wp");
            navigator.geolocation.getCurrentPosition(function() {});
        } else if (result.state == 'denied') {
            console.warn("GPS features are not enabled");
            Start_Button.remove();
            Shift_Button.remove();
            const textnode = document.createTextNode("GPS is not enabled");
            document.getElementById("map").appendChild(textnode);
            return;
        }
        init();
      });
    }

    handlePermission();*/
}()

function toggleDrive() {
    driving = !driving;
    update_buttons();

    if (Shift_Button.hidden) {
        Shift_Button.hidden = false;
        Start_Button.removeEventListener('click', toggleDrive);
        Start_Button.addEventListener('click', (event) => {
            stopDrive();
        });
    }
}

function update_buttons() {
    if (driving) {
        Start_Button.setAttribute('value','Stop Driving');
        Start_Button.classList = ["inner-input red"];

        Shift_Button.setAttribute('value','Pause Driving');
        Shift_Button.classList = ["inner-input"];
    }
    else {
        Shift_Button.setAttribute('value','Resume Driving');
        Shift_Button.classList = ["inner-input blue"];
    }
}

function haversineDistance(coords1, coords2, isMiles) {
    function toRad(x) {
      return x * Math.PI / 180;
    }
  
    var lon1 = coords1[0];
    var lat1 = coords1[1];
  
    var lon2 = coords2[0];
    var lat2 = coords2[1];
  
    var R = 6371; // km
  
    var x1 = lat2 - lat1;
    var dLat = toRad(x1);
    var x2 = lon2 - lon1;
    var dLon = toRad(x2)
    var a = Math.sin(dLat / 2) * Math.sin(dLat / 2) +
      Math.cos(toRad(lat1)) * Math.cos(toRad(lat2)) *
      Math.sin(dLon / 2) * Math.sin(dLon / 2);
    var c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
    var d = R * c;
  
    if(isMiles) d /= 1.60934;
  
    return d;
}

function calcLength() {
    var length = 0;
    for (let index = 1; index < coordinates.length; index++) {
        const element = coordinates[index];
        const past_elemnent = coordinates[index-1];
        length += haversineDistance(element, past_elemnent, false);
    }
    return length;
}

function stopDrive() {
    var passengers = parseInt(prompt('How many passengers were in the car?'));
    if (passengers.toString() == "NaN" || passengers < 1) {
        passengers = 1;
    }
    console.log(coordinates[0])
    var length = calcLength();
    var fule = length / 10; //Avg consumption of 10 km / liter
    length = Math.floor(length);

    axios.post(window.location.pathname, {
        withCredentials: true,
        
            startTime: startTime,
            endTime: Date.now(),
            length: length,
            fule: fule,
            startLocation: coordinates[0],
            endLocation: coordinates.pop(),
            passengers: passengers      
        
          })
          .then((res) => {
            alert("Success");
            window.location = window.location.pathname.replace("drive", "view");
          })
          .catch((err) => {
            alert("Failed to Submit Drive");
            window.location = window.location.pathname.replace("drive", "view");
          });
}