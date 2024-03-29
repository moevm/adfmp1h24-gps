package co.realfit.nawinitglutin;

import android.app.NativeActivity;
import android.os.Bundle;

public class MainActivity extends NativeActivity {

    static {
        System.loadLibrary("na_winit_glutin");
        LocationHelper.class.getName();
    }
    public LocationHelper locationHelper;
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        this.locationHelper = new LocationHelper(this);
        super.onCreate(savedInstanceState);
    }
}
