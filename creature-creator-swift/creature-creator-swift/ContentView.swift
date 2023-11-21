//
//  ContentView.swift
//  creature-creator-swift
//
//  Created by Edison Moreland on 11/20/23.
//

import SwiftUI

func lineSegmentTest() -> LineSegment {
    LineSegment(0, 5, 0, 0, -5, 0, 0, 0, 0, 1, 0, 0, 0)
}

struct ContentView: View {
//    var segment: LineSegment
    
    var body: some View {
        VStack {
            Image(systemName: "globe")
                .imageScale(.large)
                .foregroundStyle(.tint)
            Text("Hello, world!")
        }
        .padding()
    }
}

#Preview {
    ContentView()
}
