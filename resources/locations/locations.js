const Markers = document.querySelectorAll('.map-marker');
const colors = ["blue","gold","red","green","orange","yellow","violet","grey","black"];
var Icons = [];
var Names = new Set();
var owner = false;

!function() {
    colors.forEach(element => {
        var Icon = new L.Icon({
            iconUrl: `https://raw.githubusercontent.com/pointhi/leaflet-color-markers/master/img/marker-icon-2x-${element}.png`,
            shadowUrl: 'https://cdnjs.cloudflare.com/ajax/libs/leaflet/0.7.7/images/marker-shadow.png',
            iconSize: [25, 41],
            iconAnchor: [12, 41],
            popupAnchor: [1, -34],
            shadowSize: [41, 41]
          });
        Icons.push(Icon);
    });
}()

!function() {

    function init() {

        var map = L.map('map', {
            center: [0,0],
            zoom: 0
        });
        
        L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
        attribution: '<a href="http://openstreetmap.org/copyright">OpenStreetMap</a> contributors</a>'
        }).addTo(map);
        

        function error(err) {
        console.warn(`ERROR(${err.code}): ${err.message}`);
        }
        var group = [];
        for (let index = 0; index < Markers.length; index++) {
            // Getting the element color
            const element = Markers[index];
            var name = element.name;
            Names.add(name);
            var set_arr = [...Names];
            var inner_index = set_arr.indexOf(name) % colors.length;

            // XY
            console.log(Icons);
            var lat = element.min;
            var long = element.max;
            var marker = L.marker([lat, long], {icon: Icons[inner_index]});    // Creating a Marker
            group.push(marker);

            var btn = "";
            if (element.disabled) {
                owner = true;
                btn = `<br><button name="${lat},${long}" onclick="deleteLocation(${lat},${long})">Delete</button>`
            }
            // Adding popup to the marker
            marker.bindPopup(`<strong>Name:</strong> ${element.name}<br>
                              <strong>Description:</strong> ${element.value} ${btn}`)
                              .openPopup();
            marker.addTo(map); // Adding marker to the map
            element.remove();
            
        }
        var fit = L.featureGroup(group);
        map.fitBounds(fit.getBounds());

        if (owner && Markers.length < 10) {

            var formContent = `<form style="text-align:center" action="${window.location.pathname}" method="post" enctype="multipart/form-data">` + 
                                `Add Location<br/>` +
                                `<input class="button-inner-input type" type="text" name="type" placeholder="Name" required/><br/>` +
                                `<input class="button-inner-input type" type="text" name="description" placeholder="Description" required/><br/>` +
                                `<input class="button-inner-input" type="submit" value="Add" onclick="rel()">` +
                            `</form>`





            map.on('click', function(e) {        
                var popLocation= e.latlng;
                var popup = L.popup()
                .setLatLng(popLocation)
                .setContent(formContent)
                .openOn(map);        
            });
        }

    }

    init();
}()


function deleteLocation(lat, long) {
    axios.post(window.location.pathname, {
        withCredentials: true,
        location: [lat, long]      
        
          })
          .then((res) => {
            alert("Success");
            window.location = window.location.reload();
          })
          .catch((err) => {
            alert("Failed to Submit Drive");
            window.location = window.location.reload();
          });
}