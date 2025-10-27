// Quick script to inspect the aggregated_data.json format
const fs = require('fs');
const path = require('path');

const dataPath = path.join(__dirname, 'www', 'output', 'aggregated_data.json');

try {
    const data = fs.readFileSync(dataPath, 'utf8');
    console.log('File size:', data.length);
    console.log('\nFirst 500 characters:');
    console.log(data.substring(0, 500));
    
    // Try to parse
    const json = JSON.parse(data);
    console.log('\nParsed JSON - type:', typeof json);
    
    if (Array.isArray(json)) {
        console.log('It\'s an array with', json.length, 'items');
        if (json.length > 0) {
            console.log('\nFirst item:', JSON.stringify(json[0], null, 2));
        }
    } else if (typeof json === 'object') {
        console.log('It\'s an object');
        console.log('Keys:', Object.keys(json));
        if (json.data && Array.isArray(json.data)) {
            console.log('It has a data array with', json.data.length, 'items');
            if (json.data.length > 0) {
                console.log('\nFirst data item:', JSON.stringify(json.data[0], null, 2));
            }
        }
    }
} catch (error) {
    console.error('Error:', error.message);
}

