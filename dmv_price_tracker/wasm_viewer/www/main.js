import init, { load_and_render_map } from './pkg/wasm_viewer.js';

async function run() {
    // 1. Initialize WASM
    await init();
    console.log("WASM initialized");

    try {
        // 2. Call WASM function to get property data
        const data = await load_and_render_map();
        
        // Convert JsValue to string if needed
        const dataStr = typeof data === 'string' ? data : data.toString();
        console.log("Raw data type:", typeof data);
        console.log("Raw data (first 200 chars):", dataStr.substring(0, 200));
        
        const parsedData = JSON.parse(dataStr);
        console.log("Parsed data type:", typeof parsedData);
        console.log("Parsed data keys:", Object.keys(parsedData));
        
        // Polars might output an object with columns, not an array of objects
        let properties = [];
        if (Array.isArray(parsedData)) {
            properties = parsedData;
        } else if (parsedData.columns && Array.isArray(parsedData.columns)) {
            // Polars outputs columnar format: {columns: {lat: [...], lon: [...]}}
            // Convert to array of objects
            const cols = parsedData.columns;
            const length = cols[Object.keys(cols)[0]].length;
            
            for (let i = 0; i < length; i++) {
                const prop = {};
                for (const [key, values] of Object.entries(cols)) {
                    prop[key] = values[i];
                }
                properties.push(prop);
            }
        } else {
            properties = [parsedData];
        }
        
        console.log("Properties loaded:", properties.length);

        // 3. Create a simple map using Mapbox GL JS
        // Mapbox token is loaded from config.js (which reads from .env)
        
        if (typeof MAPBOX_ACCESS_TOKEN === 'undefined' || MAPBOX_ACCESS_TOKEN === 'YOUR_MAPBOX_ACCESS_TOKEN_HERE') {
            document.getElementById('vis').innerHTML = 
                '<h2>Error: Please set your Mapbox token in .env file</h2>' +
                '<p>1. Copy wasm_viewer/.env.example to wasm_viewer/.env</p>' +
                '<p>2. Add your Mapbox token to .env file</p>' +
                '<p>3. Run ./setup-env.sh or setup-env.bat to generate config.js</p>';
            return;
        }

        // 4. Initialize Mapbox GL
        mapboxgl.accessToken = MAPBOX_ACCESS_TOKEN;

        const map = new mapboxgl.Map({
            container: 'vis',
            style: 'mapbox://styles/mapbox/dark-v11',
            center: [-77.0369, 38.9072], // DC coordinates
            zoom: 11
        });

        // 5. Wait for map to load, then add markers for each property
        map.on('load', () => {
            console.log("Map loaded");
            
            // Add a source for the properties
            map.addSource('properties', {
                'type': 'geojson',
                'data': {
                    'type': 'FeatureCollection',
                    'features': properties.map(prop => ({
                        'type': 'Feature',
                        'geometry': {
                            'type': 'Point',
                            'coordinates': [prop.longitude, prop.latitude]
                        },
                        'properties': {
                            'title': prop.address || 'Property',
                            'price': prop.price || 'N/A',
                            'bedrooms': prop.bedrooms || 'N/A',
                            'bathrooms': prop.bathrooms || 'N/A'
                        }
                    }))
                }
            });

            // Add a layer for the properties
            map.addLayer({
                'id': 'property-points',
                'type': 'circle',
                'source': 'properties',
                'paint': {
                    'circle-radius': 6,
                    'circle-color': '#ff6b6b',
                    'circle-opacity': 0.6,
                    'circle-stroke-width': 2,
                    'circle-stroke-color': '#fff'
                }
            });

            // Add click handler to show property info
            map.on('click', 'property-points', (e) => {
                const coordinates = e.lngLat;
                const props = e.features[0].properties;
                
                new mapboxgl.Popup()
                    .setLngLat(coordinates)
                    .setHTML(
                        `<h3>${props.title}</h3>` +
                        `<p><strong>Price:</strong> $${props.price}</p>` +
                        `<p><strong>Bedrooms:</strong> ${props.bedrooms}</p>` +
                        `<p><strong>Bathrooms:</strong> ${props.bathrooms}</p>`
                    )
                    .addTo(map);
            });

            // Change cursor on hover
            map.on('mouseenter', 'property-points', () => {
                map.getCanvas().style.cursor = 'pointer';
            });

            map.on('mouseleave', 'property-points', () => {
                map.getCanvas().style.cursor = '';
            });

            console.log("Map setup complete");
        });

    } catch (error) {
        console.error("Error loading or rendering map:", error);
        document.getElementById('vis').innerHTML = 
            `<pre>Error: ${error}</pre>`;
    }
}

run();
